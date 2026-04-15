import { CodeBlock } from '@/components/code-block-with-copy';

export default function LangchainGuide() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">LangChain Integration</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Use LangChain with the platform's basic OpenAI-style chat completions endpoint.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Installation</h2>
        <CodeBlock language="bash">
          {'pip install langchain langchain-community'}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Basic Chat</h2>
        <CodeBlock language="python" title="chat.py">
          {[
            'from langchain_community.chat_models import ChatOpenAI',
            'from langchain_core.messages import HumanMessage, SystemMessage',
            '',
            '# Initialize the model',
            'llm = ChatOpenAI(',
            '    model="grok-3",',
            '    openai_api_key="your-api-key",',
            '    openai_api_base="https://api.example.com/v1",',
            '    temperature=0.7',
            ')',
            '',
            '# Chat',
            'messages = [',
            '    SystemMessage(content="You are a helpful assistant."),',
            '    HumanMessage(content="Hello!")',
            ']',
            '',
            'response = llm.invoke(messages)',
            'print(response.content)',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Chains</h2>
        <CodeBlock language="python" title="chains.py">
          {[
            'from langchain_community.chat_models import ChatOpenAI',
            'from langchain_core.prompts import ChatPromptTemplate',
            'from langchain_core.output_parsers import StrOutputParser',
            '',
            '# Create the chain',
            'llm = ChatOpenAI(',
            '    model="grok-3",',
            '    openai_api_key="your-api-key",',
            '    openai_api_base="https://api.example.com/v1"',
            ')',
            '',
            'prompt = ChatPromptTemplate.from_messages([',
            '    ("system", "You are a helpful {topic} assistant."),',
            '    ("user", "{question}")',
            '])',
            '',
            'chain = prompt | llm | StrOutputParser()',
            '',
            '# Run the chain',
            'response = chain.invoke({',
            '    "topic": "coding",',
            '    "question": "How do I reverse a string in Python?"',
            '})',
            '',
            'print(response)',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">RAG (Retrieval Augmented Generation)</h2>
        <CodeBlock language="python" title="rag.py">
          {[
            'from langchain_community.chat_models import ChatOpenAI',
            'from langchain_text_splitters import CharacterTextSplitter',
            'from langchain_community.vectorstores import FAISS',
            'from langchain.chains import RetrievalQA',
            '',
            '# Your documents',
            'texts = [',
            '    "The Grok API provides access to advanced AI models.",',
            '    "You can use streaming for real-time responses.",',
            '    "Multiple payment options are available including crypto."',
            ']',
            '',
            '# Split and embed',
            'text_splitter = CharacterTextSplitter(chunk_size=500, chunk_overlap=50)',
            'chunks = text_splitter.split_text("\\n".join(texts))',
            '',
            '# Create vector store (you will need an embedding model)',
            '# vectorstore = FAISS.from_texts(chunks, embedding)',
            '',
            '# Create QA chain',
            'llm = ChatOpenAI(',
            '    model="grok-3",',
            '    openai_api_key="your-api-key",',
            '    openai_api_base="https://api.example.com/v1"',
            ')',
            '',
            '# qa_chain = RetrievalQA.from_chain_type(',
            '#     llm=llm,',
            '#     chain_type="stuff",',
            '#     retriever=vectorstore.as_retriever()',
            '# )',
            '',
            '# response = qa_chain.invoke("What payment options are available?")',
            '# print(response["result"])',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Agent Note</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Tool-calling is not exposed by this API anymore. Use LangChain chains or plain chat flows instead of agent executors that depend on function calling.
        </p>
        <CodeBlock language="python" title="agent.py">
          {[
            'from langchain_community.chat_models import ChatOpenAI',
            'from langchain_core.messages import HumanMessage',
            '',
            '# Initialize the model',
            'llm = ChatOpenAI(',
            '    model="grok-3",',
            '    openai_api_key="your-api-key",',
            '    openai_api_base="https://api.example.com/v1",',
            '    temperature=0',
            ')',
            '',
            '# Plain invoke still works well for orchestrated app logic',
            'response = llm.invoke([HumanMessage(content="Draft a support reply for this bug report.")])',
            'print(response.content)',
          ]}
        </CodeBlock>
      </section>
    </div>
  );
}
