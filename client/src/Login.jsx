import { useState } from "react";
import { useAuth } from "./useAuth";
import { Link, useNavigate } from "react-router-dom";
import { apiUrl, readApiResponse } from "./api";

function LoginForm() {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");

  const handleLogin = async (e) => {
    e.preventDefault();
    setError("");
    try {
      const res = await fetch(apiUrl("/api/auth/login"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
      });
      const data = await readApiResponse(res);
      login(data.token);
      navigate("/");
    } catch (error) {
      setError(error.message || "Login failed");
    }
  };
  return (
    <main className="container flex h-screen items-center justify-center flex-col gap-12">
      <h2 className="text-6xl font-bold text-center">Login to your account</h2>
      <form onSubmit={handleLogin} className="flex flex-col gap-8 items-center">
        <div className="flex flex-col items-center gap-4">
          <p className="text-2xl">Enter your email:</p>
          <input
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="Email"
            className="border-1 h-15 w-120 p-5 rounded-lg text-xl"
          />
        </div>
        <div className="flex flex-col items-center gap-4">
          <p className="text-2xl">Enter your password:</p>
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="border-1 h-15 w-120 p-5 rounded-lg text-xl"
          />
        </div>
        <button
          type="submit"
          className="h-15 w-120 py-2 rounded-lg text-xl duration-250 cursor-pointer bg-red-400 text-white hover:bg-red-600"
        >
          <p>Login</p>
        </button>
      </form>

      {error && <div className="text-red-500 text-2xl">{error}</div>}

      <div className="flex items-center text-2xl gap-5">
        <p className="">Don't have an account?</p>
        <Link to="/signup" className="font-bold rounded-lg">
          <p className="text-white hover:underline">Sign up</p>
        </Link>
      </div>
    </main>
  );
}

export default LoginForm;
