export default function PrivacyPage() {
  return (
    <div className="min-h-screen bg-white dark:bg-slate-950 py-12">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        <h1 className="text-4xl font-bold mb-8">Privacy Policy</h1>

        <div className="space-y-6 text-slate-600 dark:text-slate-400">
          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">1. Information We Collect</h2>
            <p className="mb-4">
              We collect information you provide directly to us, including:
            </p>
            <ul className="list-disc list-inside space-y-2">
              <li>Account information (email, name from Google OAuth)</li>
              <li>API usage data and request logs</li>
              <li>Payment and billing information</li>
              <li>Device and browser information</li>
            </ul>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">2. How We Use Your Information</h2>
            <p className="mb-4">
              We use the collected information to:
            </p>
            <ul className="list-disc list-inside space-y-2">
              <li>Provide and maintain our API service</li>
              <li>Process payments and send invoices</li>
              <li>Monitor and prevent abuse</li>
              <li>Improve our service quality</li>
              <li>Comply with legal obligations</li>
            </ul>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">3. Data Security</h2>
            <p className="mb-4">
              We implement appropriate technical and organizational measures to protect your personal data. However, no method of transmission over the internet is 100% secure.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">4. Data Retention</h2>
            <p className="mb-4">
              We retain your personal data for as long as your account is active and for 90 days after account deletion for legal and operational purposes.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">5. Third-Party Services</h2>
            <p className="mb-4">
              We use third-party services including:
            </p>
            <ul className="list-disc list-inside space-y-2">
              <li>Google OAuth for authentication</li>
              <li>Payment processors for transactions</li>
              <li>Analytics services for usage tracking</li>
            </ul>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">6. Your Rights</h2>
            <p className="mb-4">
              You have the right to:
            </p>
            <ul className="list-disc list-inside space-y-2">
              <li>Access your personal data</li>
              <li>Correct inaccurate data</li>
              <li>Request deletion of your data</li>
              <li>Export your data</li>
              <li>Opt-out of marketing communications</li>
            </ul>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">7. Children&apos;s Privacy</h2>
            <p className="mb-4">
              Our service is not intended for children under 13. We do not knowingly collect personal information from children.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">8. Changes to Privacy Policy</h2>
            <p className="mb-4">
              We may update this privacy policy from time to time. We will notify you of any changes by posting the new policy on this page.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">9. Contact Us</h2>
            <p className="mb-4">
              For privacy-related questions, contact us at privacy@example.com
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}
