// RouteTracer.tsx (TEMP)
import { useEffect } from "react";
import { useLocation } from "react-router-dom";
export default function RouteTracer() {
  const loc = useLocation();
  useEffect(() => {
    console.log("[ROUTE]", loc.pathname, "state:", loc.state);
  }, [loc]);
  return null;
}
