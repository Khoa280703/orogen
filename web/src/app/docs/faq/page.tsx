'use client';

import { ChevronDown } from 'lucide-react';
import { useState } from 'react';

interface FAQItem {
  question: string;
  answer: string;
}

const faqItems: FAQItem[] = [
  {
    question: 'How do I get an API key?',
    answer: 'Sign up with your Google account, then navigate to your dashboard and click on "API Keys" to generate a new key. Keep your key secure and never share it publicly.',
  },
  {
    question: 'What payment methods do you accept?',
    answer: 'We accept multiple payment methods including bank transfer (Vietnam), cryptocurrency (USDT, BTC, ETH), and fpayment for international users. Check the pricing page for details.',
  },
  {
    question: 'Is there a free tier?',
    answer: 'Yes! We offer a free tier with limited requests per day. This is perfect for testing and development. Upgrade to a paid plan for higher limits and production use.',
  },
  {
    question: 'What are the rate limits?',
    answer: 'Rate limits vary by subscription plan. Free tier allows 100 requests/day, Pro tier allows 10,000 requests/day, and Enterprise has custom limits. Check your dashboard for current usage.',
  },
  {
    question: 'Can I use this for commercial projects?',
    answer: 'Yes! You can use our API for commercial projects. However, please review our Terms of Service for any restrictions on specific use cases.',
  },
  {
    question: 'How do I handle rate limit errors?',
    answer: 'When you hit the rate limit, you will receive a 429 status code. Implement exponential backoff in your application and retry after the specified time. Consider upgrading your plan for higher limits.',
  },
  {
    question: 'Do you offer refunds?',
    answer: 'Refunds are processed on a case-by-case basis. Contact support with your request and we will review it within 48 hours.',
  },
  {
    question: 'How can I contact support?',
    answer: 'You can reach our support team through the dashboard help section or email us at support@example.com. We typically respond within 24 hours.',
  },
];

export default function FAQPage() {
  const [openIndex, setOpenIndex] = useState<number | null>(null);

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Frequently Asked Questions</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Find answers to common questions about our API.
        </p>
      </div>

      <div className="space-y-4">
        {faqItems.map((item, index) => (
          <div
            key={index}
            className="border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden"
          >
            <button
              onClick={() => setOpenIndex(openIndex === index ? null : index)}
              className="flex items-center justify-between w-full p-4 text-left hover:bg-slate-50 dark:hover:bg-slate-900"
            >
              <span className="font-semibold">{item.question}</span>
              <ChevronDown
                className={`h-5 w-5 transition-transform ${
                  openIndex === index ? 'rotate-180' : ''
                }`}
              />
            </button>
            {openIndex === index && (
              <div className="px-4 pb-4 text-slate-600 dark:text-slate-400">
                {item.answer}
              </div>
            )}
          </div>
        ))}
      </div>

      <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-6 text-center">
        <h3 className="text-lg font-semibold mb-2">Still have questions?</h3>
        <p className="text-slate-600 dark:text-slate-400 mb-4">
          Contact our support team for personalized assistance.
        </p>
        <a
          href="mailto:support@example.com"
          className="text-blue-500 hover:underline"
        >
          support@example.com
        </a>
      </div>
    </div>
  );
}
