import { BrowserRouter, Route, Routes } from "react-router-dom";
import Summarizer from "./Summarizer";
import History from "./History";
import Layout from "./Layout";
import Login from "./Login";
import Signup from "./Signup";
import { useEffect } from "react";

function App() {
  useEffect(() => {
    document.title = "QuickBlog";
  }, []);

  return (
    <BrowserRouter>
      <main>
        <Layout>
          <Routes>
            <Route path="/login" element={<Login />} />
            <Route path="/signup" element={<Signup />} />
            <Route path="/" element={<Summarizer />} />
            <Route path="/history" element={<History />} />
          </Routes>
        </Layout>
      </main>
    </BrowserRouter>
  );
}

export default App;
