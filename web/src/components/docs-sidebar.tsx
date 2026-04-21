'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { ChevronDown } from 'lucide-react';
import { useState } from 'react';

interface NavItem {
  label: string;
  href?: string;
  children?: NavItem[];
}

const navItems: NavItem[] = [
  {
    label: 'Getting Started',
    href: '/docs',
  },
  {
    label: 'API Reference',
    href: '/docs/api',
  },
  {
    label: 'Guides',
    children: [
      { label: 'Quickstart', href: '/docs/guides/quickstart' },
      { label: 'Codex CLI', href: '/docs/guides/codex-cli' },
      { label: 'Provider Onboarding', href: '/docs/guides/provider-onboarding' },
      { label: 'Python', href: '/docs/guides/python' },
      { label: 'Node.js', href: '/docs/guides/nodejs' },
      { label: 'cURL', href: '/docs/guides/curl' },
      { label: 'LangChain', href: '/docs/guides/langchain' },
    ],
  },
  {
    label: 'Models',
    href: '/docs/models',
  },
  {
    label: 'FAQ',
    href: '/docs/faq',
  },
];

export function DocsSidebar() {
  const pathname = usePathname();
  const [openItems, setOpenItems] = useState<Record<string, boolean>>({
    guides: true,
  });

  const toggleItem = (label: string) => {
    setOpenItems((prev) => ({ ...prev, [label]: !prev[label] }));
  };

  return (
    <nav className="space-y-1">
      {navItems.map((item) => (
        <div key={item.href || item.label}>
          {item.children ? (
            <div>
              <button
                onClick={() => toggleItem(item.label)}
                className="flex items-center justify-between w-full px-3 py-2 text-sm font-medium rounded-md hover:bg-slate-100 dark:hover:bg-slate-800"
              >
                <span>{item.label}</span>
                <ChevronDown
                  className={`h-4 w-4 transition-transform ${
                    openItems[item.label] ? 'rotate-180' : ''
                  }`}
                />
              </button>
              {openItems[item.label] && (
                <div className="ml-4 mt-1 space-y-1">
                  {item.children.map((child) => (
                    <Link
                      key={child.label}
                      href={child.href || '#'}
                      className={`block px-3 py-2 text-sm rounded-md ${
                        pathname === child.href
                          ? 'bg-slate-100 dark:bg-slate-800 font-medium'
                          : 'hover:bg-slate-50 dark:hover:bg-slate-800/50'
                      }`}
                    >
                      {child.label}
                    </Link>
                  ))}
                </div>
              )}
            </div>
          ) : item.href ? (
            <Link
              href={item.href}
              className={`block px-3 py-2 text-sm font-medium rounded-md ${
                pathname === item.href
                  ? 'bg-slate-100 dark:bg-slate-800'
                  : 'hover:bg-slate-50 dark:hover:bg-slate-800/50'
              }`}
            >
              {item.label}
            </Link>
          ) : null}
        </div>
      ))}
    </nav>
  );
}
