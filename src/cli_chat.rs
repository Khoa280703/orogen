use std::io::{self, BufRead, Write};

use crate::account::pool::AccountPool;
use crate::config::load_config;
use crate::conversation::{Conversation, ConversationStore};
use crate::db;
use crate::grok::client::{GrokClient, GrokRequestError, StreamEvent};
use crate::grok::output_sanitizer::OutputSanitizer;
use crate::grok::types::{GrokRequest, GrokStreamEvent};

pub struct CliChat {
    pool: AccountPool,
    store: ConversationStore,
    conversation: Conversation,
    grok: GrokClient,
    model: String,
    reasoning: bool,
}

impl CliChat {
    pub async fn new(resume_id: Option<&str>) -> Self {
        let _ = dotenvy::dotenv();
        let config = load_config();
        let url = config
            .database_url
            .as_ref()
            .expect("DATABASE_URL is required. Set it in .env or environment.");
        let db = db::init_pool(url)
            .await
            .expect("Failed to connect to PostgreSQL");
        let pool = AccountPool::new(db);
        let store = ConversationStore::new();
        let grok = GrokClient::new()
            .await
            .expect("Failed to create Grok client");
        let model = config.default_model.clone();

        let conversation = if let Some(id) = resume_id {
            match store.load(id) {
                Some(conv) => conv,
                None => {
                    eprintln!("\x1b[33mConversation {id} not found. Starting new.\x1b[0m");
                    store.create(&model)
                }
            }
        } else {
            store.create(&model)
        };

        let model = conversation.model.clone();

        Self {
            pool,
            store,
            conversation,
            grok,
            model,
            reasoning: false,
        }
    }

    /// Start the interactive chat loop (blocking, runs tokio runtime internally)
    pub async fn start(&mut self) {
        self.print_welcome().await;

        let stdin = io::stdin();
        let mut reader = stdin.lock().lines();

        loop {
            // Prompt
            print!("\x1b[32m\nYou: \x1b[0m");
            io::stdout().flush().ok();

            let line = match reader.next() {
                Some(Ok(line)) => line,
                _ => break,
            };

            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            // Handle slash commands
            if trimmed.starts_with('/') {
                let should_quit = self.handle_command(&trimmed).await;
                if should_quit {
                    break;
                }
                continue;
            }

            // Send message to Grok
            self.send_message(&trimmed).await;
        }
    }

    async fn print_welcome(&self) {
        let account_name = self
            .pool
            .get_current()
            .await
            .map(|a| a.name)
            .unwrap_or_else(|| "\x1b[31mnone configured!\x1b[0m".into());

        println!("\n\x1b[1m--- Grok Local Chat ---\x1b[0m");
        println!(
            "\x1b[2mModel: {} | Reasoning: {}\x1b[0m",
            self.model,
            if self.reasoning { "on" } else { "off" }
        );
        println!("\x1b[2mConversation: {}\x1b[0m", self.conversation.id);
        println!("\x1b[2mAccount: {account_name}\x1b[0m");
        if !self.conversation.messages.is_empty() {
            println!(
                "\x1b[2mResuming with {} messages\x1b[0m",
                self.conversation.messages.len()
            );
        }
        println!("\x1b[2mCommands: /new /model /reason /accounts /history /quit /help\x1b[0m");
        println!();
    }

    /// Send user message and stream response in real-time
    async fn send_message(&mut self, content: &str) {
        let account = match self.pool.get_current().await {
            Some(a) => a,
            None => {
                eprintln!("\x1b[31mNo accounts configured! Add cookies to cookies.json\x1b[0m");
                return;
            }
        };

        // Add user message to history
        self.store
            .add_message(&mut self.conversation, "user", content);

        // Build Grok request from conversation history
        let (system_prompt, message) = flatten_messages(&self.conversation);
        let payload = GrokRequest::new(message, self.model.clone(), self.reasoning, system_prompt);

        print!("\x1b[36m\nGrok: \x1b[0m");
        io::stdout().flush().ok();

        let mut full_response = String::new();
        let mut sanitizer = OutputSanitizer::new();

        // Try streaming request
        let proxy_ref = account.proxy_url.as_ref();
        match self
            .grok
            .send_request_stream(&account.cookies, &payload, proxy_ref)
            .await
        {
            Ok(mut rx) => {
                self.pool.mark_used().await;
                self.receive_stream(&mut rx, &mut full_response, &mut sanitizer)
                    .await;
            }
            Err(GrokRequestError::RateLimited | GrokRequestError::Unauthorized) => {
                eprintln!("\x1b[33m\n[Rotating to next account...]\x1b[0m");
                if self.pool.rotate().await {
                    if let Some(next) = self.pool.get_current().await {
                        let next_proxy = next.proxy_url.as_ref();
                        match self
                            .grok
                            .send_request_stream(&next.cookies, &payload, next_proxy)
                            .await
                        {
                            Ok(mut rx) => {
                                self.pool.mark_used().await;
                                self.receive_stream(&mut rx, &mut full_response, &mut sanitizer)
                                    .await;
                            }
                            Err(e) => eprintln!("\x1b[31m\nError: {e}\x1b[0m"),
                        }
                    }
                } else {
                    eprintln!("\x1b[31m\nAll accounts exhausted\x1b[0m");
                }
            }
            Err(e) => eprintln!("\x1b[31m\nError: {e}\x1b[0m"),
        }

        println!(); // Newline after response

        // Save assistant response
        if !full_response.is_empty() {
            self.store
                .add_message(&mut self.conversation, "assistant", &full_response);
        }
    }

    /// Receive and print streaming events in real-time
    async fn receive_stream(
        &self,
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<StreamEvent>,
        full_response: &mut String,
        sanitizer: &mut OutputSanitizer,
    ) {
        while let Some(event) = rx.recv().await {
            match event {
                StreamEvent::Event(grok_event) => match grok_event {
                    GrokStreamEvent::Token(t) => {
                        let clean = sanitizer.process(&t);
                        if !clean.is_empty() {
                            print!("{clean}");
                            full_response.push_str(&clean);
                            io::stdout().flush().ok();
                        }
                    }
                    GrokStreamEvent::Thinking(t) => {
                        let clean = sanitizer.process(&t);
                        if !clean.is_empty() {
                            print!("\x1b[2m{clean}\x1b[0m");
                            io::stdout().flush().ok();
                        }
                    }
                    GrokStreamEvent::WebSearch => {
                        print!("\x1b[33m\n[Searching web...]\n\x1b[0m");
                        io::stdout().flush().ok();
                    }
                    GrokStreamEvent::Done => {}
                },
                StreamEvent::Error(e) => {
                    eprintln!("\x1b[31m\nError: {e}\x1b[0m");
                    break;
                }
                StreamEvent::Done => break,
            }
        }
    }

    /// Handle slash commands. Returns true if should quit.
    async fn handle_command(&mut self, cmd: &str) -> bool {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        match command {
            "/quit" | "/exit" => {
                println!("\x1b[2mShutting down browser...\x1b[0m");
                self.grok.shutdown().await;
                println!("\x1b[2mBye!\x1b[0m");
                return true;
            }

            "/new" => {
                self.conversation = self.store.create(&self.model);
                println!("\x1b[32mNew conversation: {}\x1b[0m", self.conversation.id);
            }

            "/model" => {
                if args.is_empty() {
                    println!("\x1b[2mCurrent: {}\x1b[0m", self.model);
                    println!("\x1b[2mAvailable: grok-3, grok-latest\x1b[0m");
                    println!("\x1b[2mUsage: /model grok-3\x1b[0m");
                } else {
                    self.model = args[0].to_string();
                    self.conversation.model = self.model.clone();
                    self.store.save(&self.conversation);
                    println!("\x1b[32mModel set to: {}\x1b[0m", self.model);
                }
            }

            "/reason" => {
                self.reasoning = !self.reasoning;
                println!(
                    "\x1b[32mReasoning mode: {}\x1b[0m",
                    if self.reasoning { "ON" } else { "OFF" }
                );
            }

            "/accounts" => {
                let account = self.pool.get_current().await;
                match account {
                    Some(a) => println!("\x1b[2mCurrent: {}\x1b[0m", a.name),
                    None => println!("\x1b[2mNo accounts configured\x1b[0m"),
                }
            }

            "/history" => {
                let convs = self.store.list();
                if convs.is_empty() {
                    println!("\x1b[2mNo conversations yet.\x1b[0m");
                } else {
                    for c in convs.iter().take(10) {
                        let marker = if c.id == self.conversation.id {
                            "\x1b[32m>\x1b[0m"
                        } else {
                            " "
                        };
                        let date = &c.updated_at[..10.min(c.updated_at.len())];
                        println!(
                            "{marker} \x1b[1m{}\x1b[0m {} ({} msgs, {date})",
                            c.id,
                            c.title,
                            c.messages.len()
                        );
                    }
                    println!("\x1b[2m\nResume: /resume <id>\x1b[0m");
                }
            }

            "/resume" => {
                if args.is_empty() {
                    println!("\x1b[2mUsage: /resume <conversation-id>\x1b[0m");
                } else {
                    match self.store.load(args[0]) {
                        Some(conv) => {
                            let msg_count = conv.messages.len();
                            let title = conv.title.clone();
                            self.model = conv.model.clone();
                            self.conversation = conv;
                            println!("\x1b[32mResumed: {title} ({msg_count} messages)\x1b[0m");
                        }
                        None => println!("\x1b[31mConversation {} not found.\x1b[0m", args[0]),
                    }
                }
            }

            "/delete" => {
                if args.is_empty() {
                    println!("\x1b[2mUsage: /delete <conversation-id>\x1b[0m");
                } else {
                    let del_id = args[0];
                    if self.store.delete(del_id) {
                        println!("\x1b[32mDeleted: {del_id}\x1b[0m");
                        if del_id == self.conversation.id {
                            self.conversation = self.store.create(&self.model);
                            println!("\x1b[2mNew conversation: {}\x1b[0m", self.conversation.id);
                        }
                    } else {
                        println!("\x1b[31mNot found: {del_id}\x1b[0m");
                    }
                }
            }

            "/help" => {
                println!(
                    "\x1b[2m{}\x1b[0m",
                    [
                        "Commands:",
                        "  /new              Start new conversation",
                        "  /model <name>     Switch model (grok-3, grok-latest)",
                        "  /reason           Toggle reasoning/thinking mode",
                        "  /accounts         Show account pool",
                        "  /history          List conversations",
                        "  /resume <id>      Resume a conversation",
                        "  /delete <id>      Delete a conversation",
                        "  /quit             Exit",
                    ]
                    .join("\n")
                );
            }

            _ => println!("\x1b[2mUnknown command: {command}. Type /help\x1b[0m"),
        }

        false
    }
}

/// Flatten conversation messages into (system_prompt, message) for Grok API
fn flatten_messages(conv: &Conversation) -> (String, String) {
    let mut system_parts = Vec::new();
    let mut chat_parts = Vec::new();

    for msg in &conv.messages {
        match msg.role.as_str() {
            "system" => system_parts.push(msg.content.as_str()),
            "assistant" => chat_parts.push(format!("[Assistant]\n{}", msg.content)),
            _ => chat_parts.push(format!("[User]\n{}", msg.content)),
        }
    }

    let system = system_parts.join("\n");
    let message = if chat_parts.len() == 1 && conv.messages.last().is_some_and(|m| m.role == "user")
    {
        conv.messages.last().unwrap().content.clone()
    } else {
        chat_parts.join("\n\n")
    };

    (system, message)
}
