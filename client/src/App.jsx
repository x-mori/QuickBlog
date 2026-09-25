import { BrowserRouter, Route, Routes } from "react-router-dom";
import Summarizer from "./Summarizer";
import History from "./History";
import Layout from "./Layout";
import RequireAuth from "./RequireAuth";
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
            <Route path="/" element={<RequireAuth><Summarizer /></RequireAuth>} />
            <Route path="/history" element={<RequireAuth><History /></RequireAuth>} />
          </Routes>
        </Layout>
      </main>
    </BrowserRouter>
  );
}

export default App;
