const baseUrl = process.env.NEXT_PUBLIC_API_URL;

let requestMore = true;
function authenticate() {
  let token = localStorage.getItem("token");

  if (!token && requestMore) {
    requestMore = false;
    token = window.prompt("Please enter your access token:");
    if (token) {
      localStorage.setItem("token", token);
    }
  }

  if (!token) {
    throw new Error("No token provided. Please refresh the page.");
  }

  return `Basic ${Buffer.from(`admin:${token}`, "utf8").toString("base64")}`;
}

async function baseRequest({ url, method, body }) {
  const authHeader = authenticate();
  const response = await fetch(`${baseUrl}${url}`, {
    method,
    headers: {
      "Content-Type": "application/json",
      Authorization: authHeader,
    },
    body: JSON.stringify(body),
  });

  if (response.status === 401) {
    localStorage.removeItem("token");
  }

  if (!response.ok) {
    throw new Error(await response.text());
  }

  return await response.json();
}

const api = {
  getStatus() {
    return baseRequest({ url: "/status", method: "GET" });
  },

  sendCommand({ intervals }) {
    return baseRequest({
      url: "/command",
      method: "POST",
      body: { intervals },
    });
  },
};

export { api };
