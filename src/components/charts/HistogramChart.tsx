import { YAxis, XAxis, CartesianGrid, BarChart, Tooltip, Bar } from "recharts";
import { HistogramBin } from "../../types/charts";

const renderHistogram = (data: HistogramBin[]) => {
  return (
    <BarChart data={data}>
      <CartesianGrid strokeDasharray="3 3" />
      <XAxis dataKey="bin" />
      <YAxis />
      <Tooltip />
      <Bar dataKey="count" fill="#8884d8" />
    </BarChart>
  );
};

export default renderHistogram;
