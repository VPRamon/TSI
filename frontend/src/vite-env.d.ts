/// <reference types="vite/client" />

declare module 'react-plotly.js' {
  import { Component } from 'react';
  import Plotly from 'plotly.js-dist-min';

  interface PlotParams {
    data: Plotly.Data[];
    layout?: Partial<Plotly.Layout>;
    config?: Partial<Plotly.Config>;
    frames?: Plotly.Frame[];
    style?: React.CSSProperties;
    className?: string;
    useResizeHandler?: boolean;
    onInitialized?: (figure: Plotly.Figure, graphDiv: HTMLElement) => void;
    onUpdate?: (figure: Plotly.Figure, graphDiv: HTMLElement) => void;
    onPurge?: (figure: Plotly.Figure, graphDiv: HTMLElement) => void;
    onError?: (error: Error) => void;
    onSelected?: (event: Plotly.PlotSelectionEvent | undefined) => void;
    onDeselect?: () => void;
    onClick?: (event: Plotly.PlotMouseEvent) => void;
    onHover?: (event: Plotly.PlotMouseEvent) => void;
    onUnhover?: (event: Plotly.PlotMouseEvent) => void;
    divId?: string;
  }

  export default class Plot extends Component<PlotParams> {}
}

declare module 'react-plotly.js/factory' {
  import { ComponentType } from 'react';
  import type Plot from 'react-plotly.js';

  // The factory accepts any Plotly module (full build or dist-min)
  // and returns a typed React component.
  export default function createPlotlyComponent(
    Plotly: unknown,
  ): typeof Plot;
}
