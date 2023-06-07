const baseUrl = process.env.NEXT_PUBLIC_API_URL;

async function baseRequest({ url, method, body }) {
  const response = await fetch(`${baseUrl}${url}`, {
    method,
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

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
