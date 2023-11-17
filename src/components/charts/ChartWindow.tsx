import { useEffect, useState } from "react";
import GenericChart from "./GenericChart";
import { events } from "../../bindings";
import ReactDOM from "react-dom/client";
import renderLineProfileChart from "./LineProfileChart";

console.log("Hello");
const ChartWindow = () => {
  const [chartData, setChartData] = useState([]);

  useEffect(() => {
    console.log("Chart Window setup");
    events.lineProfileEvent.listen((e) => {
      console.log(e.payload);
    });
  }, []);

  return (
    <div style={{ width: "100vw", height: "100vh" }}>
      <GenericChart data={chartData} renderChart={renderLineProfileChart} />
    </div>
  );
};

ReactDOM.createRoot(document.getElementById("root")!).render(<ChartWindow />);