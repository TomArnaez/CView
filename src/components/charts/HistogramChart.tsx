import { YAxis, XAxis, CartesianGrid, BarChart, Tooltip, Bar } from "recharts";
import { HistogramBin } from "../../bindings";


const renderHistogram = (histogramData: HistogramBin[]) => {
  return (
    <BarChart data={histogramData}>
      <XAxis dataKey="range" domain={["auto", "auto"]} interval={32}/>
      <YAxis />
      <Tooltip />
      <Bar dataKey="count" fill="#8884d8" />
    </BarChart>
  );
};

export default renderHistogram;