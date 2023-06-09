import { api } from "../lib/api";
import { useEffect, useState } from "react";

export const StatusBage = () => {
  const [{ isOnline, error }, setIsOnline] = useState({});
  useEffect(() => {
    async function getStatus() {
      try {
        let { online } = await api.getStatus();
        setIsOnline({ isOnline: online });
      } catch (error) {
        setIsOnline({ isOnline: false, error });
      }
    }
    getStatus();
    const h = setInterval(getStatus, 3000);
    return () => {
      clearInterval(h);
    };
  }, []);

  if (error) {
    return (
      <span style={{ color: "darkred", fontWeight: "bold" }}>
        {(error && error.message) || error}
      </span>
    );
  }

  if (isOnline === undefined) {
    return (
      <span style={{ color: "darkgray", fontWeight: "bold" }}>
        checking status...
      </span>
    );
  }

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
