export default function TermsPage() {
  return (
    <div className="min-h-screen bg-white dark:bg-slate-950 py-12">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        <h1 className="text-4xl font-bold mb-8">Terms of Service</h1>

        <div className="space-y-6 text-slate-600 dark:text-slate-400">
          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">1. Acceptance of Terms</h2>
            <p className="mb-4">
              By accessing or using our API service, you agree to be bound by these Terms of Service. If you disagree with any part of these terms, you may not use our service.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">2. Service Description</h2>
            <p className="mb-4">
              We provide API access to AI models for various use cases. The service is provided as-is and we reserve the right to modify or discontinue the service at any time.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">3. Account and API Keys</h2>
            <p className="mb-4">
              You are responsible for maintaining the security of your API key. You must notify us immediately of any unauthorized use of your API key. We are not liable for any losses resulting from unauthorized use.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">4. Acceptable Use</h2>
            <p className="mb-4">
              You agree not to use our service for:
            </p>
            <ul className="list-disc list-inside space-y-2">
              <li>Illegal activities or content</li>
              <li>Generating hate speech or harassment</li>
              <li>Creating malware or conducting cyberattacks</li>
              <li>Spam or unsolicited bulk messages</li>
              <li>Any activity that violates applicable laws</li>
            </ul>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">5. Payment and Billing</h2>
            <p className="mb-4">
              You agree to pay all fees associated with your account. All payments are non-refundable unless required by law. We reserve the right to adjust pricing with 30 days notice.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">6. Limitation of Liability</h2>
            <p className="mb-4">
              Our service is provided &quot;as is&quot; without warranties of any kind. We are not liable for any indirect, incidental, or consequential damages arising from your use of the service.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">7. Changes to Terms</h2>
            <p className="mb-4">
              We may update these terms at any time. Continued use of the service after changes constitutes acceptance of the new terms.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold mb-4 text-slate-900 dark:text-white">8. Contact</h2>
            <p className="mb-4">
              For questions about these terms, contact us at support@example.com
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}
