import { api } from "../lib/api";
import { useEffect, useState } from "react";

export const StatusBage = () => {
  const [isOnline, setIsOnline] = useState(false);
  useEffect(() => {
    const h = setInterval(async () => {
      try {
        let { online } = await api.getStatus();
        setIsOnline(online);
      } catch (error) {
        setIsOnline(false);
      }
    }, 3000);
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
