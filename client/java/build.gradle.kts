plugins {
    id("java")
    id("java-library")
    id("maven-publish")
}

group = "dev.lunna.daiana"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    api(libs.bundles.netty.websocket)
    api(libs.jetbrains.annotations)

    testImplementation(platform("org.junit:junit-bom:6.0.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
}