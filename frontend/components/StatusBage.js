import { api } from "../lib/api";
import { useEffect, useState } from "react";

export const StatusBage = () => {
  const [isOnline, setIsOnline] = useState(false);
  useEffect(() => {
    async function getStatus() {
        try {
            let { online } = await api.getStatus();
            setIsOnline(online);
        } catch (error) {
            setIsOnline(false);
        }
    }
    getStatus();
    const h = setInterval(getStatus, 3000);
    return () => {
      clearInterval(h);
    };
  }, []);

  if (isOnline) {
    return (
      <span style={{ color: "darkgreen", fontWeight: "bold" }}>Online</span>
    );
  } else {
    return (
      <span style={{ color: "darkred", fontWeight: "bold" }}>Offline</span>
    );
  }
};
