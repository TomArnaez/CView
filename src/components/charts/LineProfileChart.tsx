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
  <LineChart margin={{ top: 5, right: 30, left: 20, bottom: 5 }} data={data}>
    <CartesianGrid strokeDasharray="3 3" />
    <XAxis dataKey="idx" />
    <YAxis />
    <Tooltip />
    <Line
      type="monotone"
      dataKey="value"
      stroke="#8884d8"
      animationDuration={300}
    />
  </LineChart>
);

export default renderLineProfileChart;
