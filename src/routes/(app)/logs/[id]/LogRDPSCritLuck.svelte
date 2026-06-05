<script lang="ts">
  import { chartable, type EChartsOptions } from "$lib/charts";
  import Card from "$lib/components/Card.svelte";
  import type { ContributionSplit } from "$lib/types";
  import { abbreviateNumber } from "$lib/utils";
  import tinygradient from "tinygradient";

  const redGreenGradient = tinygradient("#FF85A9", "#5794F2", "#2DC88F");

  interface Props {
    split: ContributionSplit;
  }

  let { split }: Props = $props();

  function zToProbability(z: number) {
    // thanks claude (wtf is this)
    const t = 1 / (1 + 0.2316419 * Math.abs(z));
    const d = 0.3989423 * Math.exp((-z * z) / 2);
    let p = d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));
    return z > 0 ? 1 - p : p;
  }

  function generateDamageDistribution(expectedDamage: number, variance: number, numPoints = 101) {
    const stdDev = Math.sqrt(variance);
    if (stdDev <= 0) return []; // No variance, no curve

    // Span 4 standard deviations in both directions (covers 99.99% of outcomes)
    const minX = expectedDamage - 4.0 * stdDev;
    const maxX = expectedDamage + 4.0 * stdDev;
    const step = (maxX - minX) / (numPoints - 1);

    const normalizationFactor = 1.0 / Math.sqrt(variance * 2.0 * Math.PI);
    const points = [];

    for (let i = 0; i < numPoints; i++) {
      const x = minX + i * step;
      const exponent = -0.5 * Math.pow((x - expectedDamage) / stdDev, 2);
      const y = normalizationFactor * Math.exp(exponent);

      points.push([x, y, i]);
    }

    return points;
  }

  const actualTotalDamage = $derived(split.damageDoneWithoutCrits + split.damageDoneFromCrits!);
  const expectedTotalDamage = $derived(split.damageDoneWithAverageCrits);
  const damageDiff = $derived(actualTotalDamage - expectedTotalDamage);
  const stdDev = $derived(Math.sqrt(split.damageDoneVariance ?? 0));
  const zScore = $derived(stdDev > 0 ? (actualTotalDamage - expectedTotalDamage) / stdDev : 0);
  const roughProbability = $derived(zToProbability(zScore));

  const luckClass = $derived.by(() => {
    if (roughProbability < 0.05) return ["Extremely Unlucky", "text-red-500"];
    if (roughProbability < 0.25) return ["Very Unlucky", "text-orange-500"];
    if (roughProbability < 0.4) return ["Somewhat Unlucky", "text-yellow-500"];
    if (roughProbability < 0.6) return ["Average", "text-neutral-300"];
    if (roughProbability < 0.75) return ["Somewhat Lucky", "text-green-400"];
    if (roughProbability < 0.95) return ["Very Lucky", "text-green-500"];
    return ["Extremely Lucky", "text-green-600"];
  });

  const distributionOptions: EChartsOptions = $derived.by((): EChartsOptions => {
    const curveData = generateDamageDistribution(expectedTotalDamage, split.damageDoneVariance ?? 0, 101);

    return {
      textStyle: {
        fontFamily: "Inter Variable"
      },
      grid: {
        left: "4%",
        right: "4%",
        top: "20%",
        bottom: "16%"
      },
      tooltip: {
        trigger: "axis",
        formatter: (params: any) => {
          const xValue = params[0].value[0];
          const percentile = params[0].value[2];
          return `${percentile.toFixed(0)}% of runs do less than ${Math.round(xValue).toLocaleString()} damage`;
        }
      },
      xAxis: {
        type: "value",
        scale: true, // Prevents axis from starting at 0
        axisLabel: {
          formatter: (value: number) => abbreviateNumber(value, 0)
        },
        splitLine: { show: false }
      },
      yAxis: {
        type: "value",
        show: false, // Hide the Y axis entirely!
        max: "dataMax" // Ensure the peak of the curve hits the top of the chart
      },
      series: [
        {
          type: "line",
          smooth: 0.6,
          symbol: "none",
          lineStyle: {
            color: "#5470C6",
            width: 3
          },
          areaStyle: {
            opacity: 0.4 // Fills the area under the curve
          },
          markLine: {
            symbol: ["none", "none"],
            label: {
              color: "#fff",
              show: true,
              formatter: (params: any) => params.name
            },
            data: [
              {
                name: "Expected Damage",
                xAxis: expectedTotalDamage,
                lineStyle: { type: "dashed", color: "#838fa1", width: 2 },
                label: { distance: 8, position: "insideMiddleTop", color: "#838fa1" }
              },
              {
                name: "Your Damage",
                xAxis: actualTotalDamage,
                lineStyle: { type: "solid", color: "#10b981", width: 2 },
                label: { distance: 8 }
              }
            ]
          },
          data: curveData
        }
      ]
    };
  });
</script>

<Card class="mt-4">
  <div class="flex items-center justify-between bg-black/10 px-3 py-2 font-medium">
    <div>Crit Luck</div>
  </div>

  <div class="flex flex-col gap-2 p-2">
    <div class="flex flex-col items-start gap-1">
      <span class="text-sm text-neutral-300">
        Your crits were <span class="font-semibold {luckClass[1]}">
          {luckClass[0]}
        </span>
        this fight (top
        <span style="color: {redGreenGradient.rgbAt(roughProbability).toHexString()}">
          {((1 - roughProbability) * 100).toFixed(0)}%
        </span>). You did {abbreviateNumber(Math.abs(damageDiff))} ({(
          (Math.abs(damageDiff) / expectedTotalDamage) *
          100
        ).toFixed(2)}%) {damageDiff >= 0 ? "more" : "less"} damage than expected.
      </span>

      <div class="h-[270px] w-full" use:chartable={distributionOptions}></div>
    </div>
  </div>
</Card>
