import ReactDOM from "react-dom/client";
import {
  LineChart,
  Line,
  YAxis,
  XAxis,
  Label,
  CartesianGrid,
  ResponsiveContainer,
} from "recharts";
import { useEffect, useState } from "react";
import { events } from "../../bindings";

const ProfileChart = (): JSX.Element => {
  const [chartData, setChartData] = useState<[]>();

  useEffect(() => {
    console.log("Line Profile Window setup");
    events.lineProfileEvent.listen((e) => {
      console.log(e);
      const chartData = e.payload.map((item) => {
        return {
          value: item[0],
          index: item[1],
        };
      });
      setChartData(chartData);
    });
  }, []);

  return (
    <div style={{ width: "95vw", height: "95vh" }}>
      <ResponsiveContainer>
        <LineChart
          data={chartData}
          margin={{
            top: 20,
            right: 10,
            left: 30,
            bottom: 10,
          }}
        >
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey={"index"} tickCount={100}>
            <Label value="X value" position="insideBottom" offset={0} dy={10} />
          </XAxis>
          <YAxis domain={["auto", "auto"]}>
            <Label
              value="ADU"
              position="insideLeft"
              offset={0}
              dx={-10}
              angle={-90}
            />
          </YAxis>
          <Line
            type="monotone"
            dataKey="value"
            isAnimationActive={false}
            stroke="#8884d8"
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};

ReactDOM.createRoot(document.getElementById("root")!).render(<ProfileChart />);
