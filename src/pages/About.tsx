declare const __APP_VERSION__: string;

export default function About() {
  return (
    <>
      <header className="page-header">
        <div>
          <h1>About</h1>
          <p>HostBlocker — block websites by managing /etc/hosts.</p>
        </div>
      </header>
      <section className="card">
        <div className="about-body">
          <p>
            <strong>Version</strong> {typeof __APP_VERSION__ === "string" ? __APP_VERSION__ : "dev"}
          </p>
          <p>
            <strong>Repository</strong>{" "}
            <a href="https://github.com/tobrun/host-blocker">github.com/tobrun/host-blocker</a>
          </p>
          <p className="note">
            Note: browsers with DNS-over-HTTPS enabled (Firefox, Chrome with secure
            DNS) may resolve names outside of <code>/etc/hosts</code> and can bypass
            these blocks.
          </p>
        </div>
      </section>
    </>
  );
}
