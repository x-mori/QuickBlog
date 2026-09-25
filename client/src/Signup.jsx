import { useState } from "react";
import { useAuth } from "./useAuth";
import { Link, useNavigate } from "react-router-dom";
import { apiUrl, readApiResponse } from "./api";

export default function Signup() {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");

  const handleSignup = async (e) => {
    e.preventDefault();
    setError("");
    try {
      const res = await fetch(apiUrl("/api/auth/signup"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
      });
      const data = await readApiResponse(res);
      login(data.token);
      navigate("/");
    } catch (error) {
      setError(error.message || "Signup failed");
    }
  };
  return (
    <main className="container flex h-screen items-center justify-center flex-col gap-12">
      <h2 className="text-6xl font-bold text-center">Sign up an account</h2>
      <form
        onSubmit={handleSignup}
        className="flex flex-col gap-8 items-center"
      >
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
          <p>Sign up</p>
        </button>
      </form>

      {error && <div className="text-red-500 text-2xl">{error}</div>}

      <div className="flex items-center text-2xl gap-5">
        <p className="">Have an account?</p>
        <Link to="/login" className="font-bold rounded-lg">
          <p className="text-white hover:underline">Login</p>
        </Link>
      </div>
    </main>
  );
}
