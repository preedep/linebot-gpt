# -*- mode: dockerfile -*-
#
# An example Dockerfile showing how to build a Rust executable using this
# image, and deploy it with a tiny Alpine Linux container.

# You can override this `--build-arg BASE_IMAGE=...` to use different
# version of Rust or OpenSSL.

#ARG BASE_IMAGE=rust:1.65.0-slim-buster
ARG BASE_IMAGE=rust:alpine

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

RUN apk update
RUN apk add --no-cache openssl-dev musl-dev perl build-base

# Add our source code.
ADD --chown=rust:rust . ./

# Build our application.
RUN cargo install --path .


# Now, we need to build our _real_ Docker container, copying in `using-diesel`.
FROM alpine:latest

ARG APP=/myapp

EXPOSE 8080

ENV TZ=Asia/Bangkok \
    APP_USER=appuser \
    RUST_LOG="debug"

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER \
    && mkdir -p ${APP}


RUN apk update \
  && apk --no-cache add ca-certificates \
  && apk add curl openssl-dev libc-dev zlib-dev libc6-compat supervisor\
  && rm -rf /var/cache/apk/*ls


RUN openssl s_client -connect southeastasia-1.in.applicationinsights.azure.com:443 -showcerts </dev/null 2>/dev/null | sed -e '/-----BEGIN/,/-----END/!d' | tee "/usr/local/share/ca-certificates/ca.crt" >/dev/null && \
update-ca-certificates

COPY --from=builder /usr/local/cargo/bin/line_botx ${APP}/line_botx.linux

#COPY chatgptproxy.linux ${APP}/chatgptproxy.linux
COPY supervisord.conf /etc/supervisord.conf

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

#ENTRYPOINT ["./line_botx"]
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisord.conf"]