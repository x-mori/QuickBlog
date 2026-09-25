import { useAuth } from "./useAuth";
import Header from "./Header";

const Layout = ({ children }) => {
  const { user } = useAuth();

  return (
    <div className="flex w-full px-20 max-md:max-w-screen min-w-screen min-h-screen flex-col gap-10 justify-center overflow-x-hidden">
      {/* header */}
      {user && <Header />}

      {/* main */}
      <main className="min-h-screen container mx-auto px-4 py-8">
        {children}
      </main>

      {/* footer */}
      <footer className="border-t rounded-t-4xl">
        <div className="container mx-auto py-4 px-6 text-end opacity-50 max-md:text-center max-md:text-sm text-2xl">
          <p>&copy; {new Date().getFullYear()} made by fujimori_</p>
        </div>
      </footer>
    </div>
  );
};

export default Layout;
