import {
  CartesianGrid,
  Line,
  LineChart,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { LineProfileData } from "../../bindings";

const renderLineProfileChart = (data: LineProfileData[]) => (
  <LineChart margin={{ top: 50, right: 30, left: 50, bottom: 20 }} data={data}>
    <CartesianGrid strokeDasharray="3 3" />
    <XAxis
      dataKey="idx"
      label={{
        value: "X-Coordinate",
        position: "insideBottom",
        offset: -15,
        style: { fontSize: "18px", fontWeight: "bold" },
     }}
      type="number"
      domain={["auto", "auto"]}
    />
    <YAxis
      label={{
        value: "Column Average",
        angle: -90,
        position: "insideLeft",
        offset: -15,
        style: { fontSize: "18px", fontWeight: "bold" },
      }}
      domain={["auto", "auto"]}
    />
    <Tooltip 
      formatter={(value: number) => [`${Math.round(value * 100) / 100}`, 'Column Average:']} 
      labelFormatter={(label) => `X-Coordinate: ${label}`}
    />  
    <Line
      type="monotone"
      dataKey="value"
      stroke="#8884d8"
      animationDuration={300}
      dot={false}
    />
  </LineChart>
);

export default renderLineProfileChart;
