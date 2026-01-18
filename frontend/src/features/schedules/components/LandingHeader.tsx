/**
 * Landing page header component.
 */
export interface LandingHeaderProps {
  title?: string;
  subtitle?: string;
}

function LandingHeader({
  title = 'Telescope Scheduling',
  subtitle = 'Upload and analyze astronomical observation schedules with precision',
}: LandingHeaderProps) {
  return (
    <header className="mb-16 text-center sm:mb-20">
      <h1 className="mb-4 text-5xl font-bold tracking-tight text-white sm:text-6xl lg:text-7xl">
        {title}
        <span className="mt-2 block bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
          Intelligence
        </span>
      </h1>
      <p className="mx-auto mt-6 max-w-2xl text-lg text-slate-400 sm:text-xl">{subtitle}</p>
    </header>
  );
}

export default LandingHeader;
