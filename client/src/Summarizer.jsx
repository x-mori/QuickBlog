import { useState } from "react";
import { Copy } from "lucide-react";
import { apiUrl, readApiResponse } from "./api";

function Summarizer() {
  const [article, setArticle] = useState("");
  const [summary, setSummary] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError("");
    setSummary("");
    setLoading(true);
    try {
      const res = await fetch(apiUrl("/api/summarize"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${localStorage.getItem("token")}`,
        },
        body: JSON.stringify({ article }),
      });
      const data = await readApiResponse(res);
      setSummary(data.summary);
    } catch (error) {
      setError(error.message || "Summarization failed");
    } finally {
      setLoading(false);
    }
  };
  return (
    <main className="container flex flex-col items-center gap-20 w-full">
      <h2 className="text-6xl font-bold text-center">QuickBlog</h2>

      {/* separator */}
      <div className="grid grid-cols-2 gap-20 max-xl:flex max-xl:flex-col">
        <div className="flex flex-col gap-5 items-center">
          <h2 className="text-4xl font-bold">Prompt</h2>
          {/* form creator */}
          <form className="flex flex-col gap-4 w-full max-md:w-[300px]" onSubmit={handleSubmit}>
            <textarea
              rows="6"
              cols="42"
              className="text-2xl border-2 border-gray-300 rounded-md p-5"
              value={article}
              onChange={(e) => setArticle(e.target.value)}
              placeholder="Paste your blog/article here..."
            />
            <button
              disabled={loading}
              className="h-13 text-2xl w-auto mt-4 bg-red-400 hover:bg-red-600 cursor-pointer duration-500 rounded-xl"
              type="submit"
            >
              <p>{loading ? "Summarizing..." : "Summarize"}</p>
            </button>
          </form>
          {error && <p role="alert" className="text-red-400">{error}</p>}
        </div>
        <div className="flex flex-col gap-5 items-center">
          <h2 className="text-4xl font-bold">Summary</h2>

          <div className="text-xl h-59 w-144 max-md:w-[300px] overflow-auto border-2 border-gray-300 rounded-md p-5">
            <p>{loading ? "Loading..." : summary}</p>
            {!summary && !loading && (
              <p className="text-2xl opacity-50">Your summary appears here..</p>
            )}
          </div>

          {/* buttons */}
          <div className="flex gap-8 mt-4">
            <button
              onClick={() => {
                navigator.clipboard.writeText(summary);
                alert("Summary copied to clipboard!");
              }}
              className="h-13 text-2xl w-auto px-4 border-2 hover:text-red-400 hover:border-red-400 cursor-pointer duration-250 rounded-xl"
            >
              <Copy />
            </button>
          </div>
        </div>
      </div>
    </main>
  );
}

export default Summarizer;
