# TSI Frontend (Vue 3 + TypeScript)

Modern, production-ready frontend for telescope scheduling analytics.

## Features

- Vue 3 with Composition API
- TypeScript for type safety
- Axios for API communication
- Minimal Tailwind-like styling
- ECharts placeholder for future chart integration

## Running Locally

```bash
cd frontend
npm install
npm run dev
```

Frontend runs on `http://localhost:5173` and proxies API requests to the backend.

## Building for Production

```bash
npm run build
npm run preview
```

## Testing

```bash
npm test
```

## Docker

```bash
docker build -t tsi-frontend .
docker run -p 80:80 tsi-frontend
```
