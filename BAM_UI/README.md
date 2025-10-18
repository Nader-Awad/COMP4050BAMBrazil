# BAM UI (React + Vite)

Frontend for the Bioscope Allocation & Management system. It consumes the Rust API exposed by `api-server/` and provides:

- Authentication flow (login, token refresh, logout) backed by JWTs stored in localStorage.
- Booking dashboard with calendar, schedule, approval controls, analytics, and live CRUD operations via the shared `apiClient` wrapper.
- Loading/error state management that surfaces API failures in the UI.

## Environment

The app reads configuration from Vite env variables:

| Variable | Default | Purpose |
| -------- | ------- | ------- |
| `VITE_API_URL` | `http://localhost:3000` | Base URL for the backend API. |

Create `.env` (or copy `.env.example`) if you need to override the defaults.

## Scripts

```bash
npm install          # install dependencies
npm run dev          # start Vite dev server on http://localhost:5173
npm run build        # build production bundle (used by Docker image)
npm run preview      # preview production build locally
```

## Docker Notes

The Dockerfile builds a static bundle and serves it via Nginx. The build argument `VITE_API_URL` (wired in `docker-compose.yml`) ensures the generated assets call the correct backend host.

## Login Credentials

The backend seeds demo accounts when migrations run:

- `admin@bam.edu` / `admin123`
- `teacher@bam.edu` / `teacher123`
- `student@bam.edu` / `student123`

Log in with any of these to explore the UI; tokens auto-refresh and are cleared on logout.
