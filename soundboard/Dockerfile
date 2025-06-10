FROM alpine:latest

COPY soundboard.exe /app/soundboard_exe

RUN if [ ! -f /app/soundboard_exe ]; then echo "soundboard.exe not found"; exit 1; fi

ARG USER_ID=4237
ARG GROUP_ID=4237
ARG DOCKER_USER=hxi2soundboard

RUN addgroup -g $GROUP_ID -S $DOCKER_USER && adduser -D -u $USER_ID -G $DOCKER_USER -S $DOCKER_USER
RUN chown -R $DOCKER_USER:$DOCKER_USER /app
USER $DOCKER_USER

ENTRYPOINT ["/app/soundboard_exe"]
