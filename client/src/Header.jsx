import { Link, useNavigate } from "react-router-dom";
import { useAuth } from "./useAuth";
import { Search, History, LogOut } from "lucide-react";

const Header = () => {
  const { logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate("/login");
  };

  return (
    <main className="sticky text-2xl max-md:text-sm flex top-0 z-50 bg-[#242424] border-b border-gray-400 py-2 mb-20 max-md:mb-10 rounded-b-4xl">
      <div className="container flex h-16 items-center justify-center px-6">
        <div className="flex gap-8 max-md:gap-2">
          <Link
            to="/"
            className="font-bold flex gap-3 items-center py-2 px-4 rounded-lg"
          >
            <Search className="text-white max-md:block max-md:mt-2" />
            <p className="text-white max-md:hidden hover:underline">
              Generator
            </p>
          </Link>
          <Link
            to="/history"
            className="font-bold flex gap-3 items-center py-2 px-4 rounded-lg"
          >
            <History className="text-white max-md:block max-md:mt-2" />
            <p className="text-white max-md:hidden hover:underline">History</p>
          </Link>

          <button
            onClick={handleLogout}
            className="font-bold flex gap-3 cursor-pointer items-center py-2 px-4 rounded-lg"
          >
            <LogOut className="text-white max-md:block max-md:mt-2" />
            <p className="text-white max-md:hidden hover:underline">Logout</p>
          </button>
        </div>
      </div>
    </main>
  );
};

export default Header;
