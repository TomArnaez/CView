import { useEffect, useState } from "react";
import { Chart, ChartData, events } from "../../bindings";
import ReactDOM from "react-dom/client";
import renderLineProfileChart from "./LineProfileChart";
import { Window } from "@tauri-apps/api/window";
import renderHistogram from "./HistogramChart";
import { ResponsiveContainer } from "recharts";

const ChartWindow = () => {
  const [chartData, setChartData] = useState([]);
  const [chartType, setChartType] = useState<Chart>();

  useEffect(() => {
    console.log("Chart Window setup for window", Window.getCurrent().label);
    events.chartDataEvent.listen((e) => {
      const data = e.payload;
      const window = e.windowLabel;
      if (window == Window.getCurrent().label) {
        if ("LineProfileData" in data) {
          setChartData(data.LineProfileData);
          setChartType("LineProfile")
        } else if ("HistogramData" in data) {
          console.log("histogram");
          setChartData(data.HistogramData);
          setChartType("Histogram");
        }
      }
    });
  }, []);

  const getChartData = (chart: Chart, chartData: ChartData): JSX.Element => {
    console.log("Hi", chartType)
    switch (chart) { 
      case "LineProfile": 
        return renderLineProfileChart(chartData)
      case "Histogram":
        return renderHistogram(chartData)
      default:
        return <></>
    }
  }

  return (
    <div style={{ width: "100vw", height: "100vh" }}>
    <ResponsiveContainer width="90%" height="90%">
      {getChartData(chartType, chartData)}
    </ResponsiveContainer>    
    </div>
  );
};

ReactDOM.createRoot(document.getElementById("root")!).render(<ChartWindow />);
