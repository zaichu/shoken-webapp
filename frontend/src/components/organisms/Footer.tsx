const REPO_BASE = 'https://github.com/zaichu/shoken-webapp/blob/main/docs';

const LINKS = [
  { label: 'プライバシーポリシー', href: `${REPO_BASE}/privacy-policy.md` },
  { label: '利用規約', href: `${REPO_BASE}/terms.md` },
  { label: 'Cookie ポリシー', href: `${REPO_BASE}/cookie-policy.md` },
] as const;

export function Footer() {
  return (
    <footer className="mt-auto border-t border-slate-200 bg-white py-4">
      <div className="mx-auto w-full max-w-[1440px] px-4 sm:px-5 lg:px-6 flex flex-wrap justify-center gap-x-6 gap-y-1 text-sm text-slate-500">
        {LINKS.map(({ label, href }) => (
          <a
            key={href}
            href={href}
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-slate-700 hover:underline"
          >
            {label}
          </a>
        ))}
        <span>© 2026 shoken-webapp</span>
      </div>
    </footer>
  );
}
