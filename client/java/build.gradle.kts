plugins {
    id("java")
    id("java-library")
    id("maven-publish")
    id("signing")
}

group = "dev.lunna.daiana4j"
version = "1.0.0"

java {
    toolchain { languageVersion = JavaLanguageVersion.of(25) }
    withSourcesJar()
    withJavadocJar()
}

repositories {
    mavenCentral()
}

dependencies {
    api(libs.bundles.netty.websocket)
    compileOnly(libs.jetbrains.annotations)
    implementation(libs.slf4j.api)

    testImplementation(platform("org.junit:junit-bom:6.0.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
}

publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])

            pom {
                name.set("Daiana")
                description.set("High-performance, asynchronous Java client for the Daiana room-based binary WebSocket relay server, powered by Netty.")
                url.set("https://github.com/Lunna5/daiana")

                licenses {
                    license {
                        name.set("GNU Affero General Public License v3.0")
                        url.set("https://www.gnu.org/licenses/agpl-3.0.html")
                    }
                }
                developers {
                    developer {
                        id.set("Lunna5")
                        name.set("Lunna Martín González")
                        email.set("git@lunna.dev")
                    }
                }
                scm {
                    connection.set("scm:git:git://github.com/Lunna5/daiana.git")
                    developerConnection.set("scm:git:ssh://github.com/Lunna5/daiana.git")
                    url.set("https://github.com/Lunna5/daiana")
                }
            }
        }
    }
}

signing {
    sign(publishing.publications["mavenJava"])
}