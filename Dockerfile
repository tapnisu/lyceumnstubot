FROM node:20-alpine AS base
LABEL authors="tapnisu"

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

WORKDIR /app

COPY package.json pnpm-lock.yaml /app/
RUN npm install --global corepack@latest && corepack enable && corepack prepare --activate

FROM base AS prod-deps
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod --frozen-lockfile

FROM base AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
COPY . /app
RUN pnpm run build

FROM base
COPY --from=build /app/dist /app/dist
COPY --from=prod-deps /app/node_modules /app/node_modules

CMD [ "pnpm", "run", "start" ]
