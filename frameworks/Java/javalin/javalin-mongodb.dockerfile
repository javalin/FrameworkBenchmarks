FROM maven:3.8.6-eclipse-temurin-19 as maven
WORKDIR /javalin
COPY src src
COPY pom.xml pom.xml
RUN mvn clean package -q

FROM amazoncorretto:19-alpine-jdk
WORKDIR /javalin
COPY --from=maven /javalin/target/javalin-1.0-shaded.jar app.jar

ARG BENCHMARK_ENV

ENV BENCHMARK_ENV=$BENCHMARK_ENV

EXPOSE 8080

CMD ["java", "-server", "-XX:+UseNUMA", "-XX:+UseParallelGC", "-Dlogging.level.root=OFF", "--enable-preview", "-jar", "app.jar"]