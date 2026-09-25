import { useState } from "react";
import { AuthContext } from "./auth-context";
import { jwtDecode } from "jwt-decode";

function storedUser() {
  const token = localStorage.getItem("token");
  if (!token) return null;
  try {
    const decoded = jwtDecode(token);
    if (!decoded.userId || !decoded.exp || decoded.exp * 1000 <= Date.now()) {
      throw new Error("Token expired");
    }
    return { id: decoded.userId };
  } catch {
    localStorage.removeItem("token");
    return null;
  }
}

export function AuthProvider({ children }) {
  const [user, setUser] = useState(storedUser);

  const login = (token) => {
    const decoded = jwtDecode(token);
    if (!decoded.userId || !decoded.exp || decoded.exp * 1000 <= Date.now()) {
      throw new Error("Invalid token");
    }
    localStorage.setItem("token", token);
    setUser({ id: decoded.userId });
  };

  const logout = () => {
    localStorage.removeItem("token");
    setUser(null);
  };

  return (
    <AuthContext.Provider value={{ user, isAuthenticated: Boolean(user), login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}
