import { ChartData } from "../../types/charts";
import { ResponsiveContainer } from "recharts";

interface ChartProps<T> {
  data: ChartData<T>;
  renderChart: (data: ChartData<T>) => JSX.Element;
}

const GenericChart = <T,>({ data, renderChart }: ChartProps<T>) => {
  return (
    <ResponsiveContainer width="90%" height="90%">
      {renderChart(data)}
    </ResponsiveContainer>
  );
};

export default GenericChart;
