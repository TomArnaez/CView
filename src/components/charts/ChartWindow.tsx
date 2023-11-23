import { useEffect, useState } from "react";
import GenericChart from "./GenericChart";
import { events } from "../../bindings";
import ReactDOM from "react-dom/client";
import renderLineProfileChart from "./LineProfileChart";
import { Window } from "@tauri-apps/api/window";
import renderHistogram from "./HistogramChart";

const ChartWindow = () => {
  const [chartData, setChartData] = useState([]);
  const [renderChartFunc, setRenderChartFunc] = useState<any>();

  useEffect(() => {
    console.log("Chart Window setup for window", Window.getCurrent().label);
    events.chartDataEvent.listen((e) => {
      const data = e.payload;
      if ("LineProfileData" in data) {
        setChartData(data.LineProfileData);
        setRenderChartFunc(renderLineProfileChart);
        console.log(renderLineProfileChart);
      } else if ("HistogramData" in chartData) {
        setChartData(data.HistogramData);
        setRenderChartFunc(renderHistogram);
        console.log(data.HistogramData);
      }
    });
    events.lineProfileEvent.listen((e) => {
      if (Window.getCurrent().label === e.windowLabel) {
        setChartData(e.payload);
      }
    });
  }, []);

  return (
    <div style={{ width: "100vw", height: "100vh" }}>
      <GenericChart data={chartData} renderChart={renderLineProfileChart} />
    </div>
  );
};

ReactDOM.createRoot(document.getElementById("root")!).render(<ChartWindow />);
