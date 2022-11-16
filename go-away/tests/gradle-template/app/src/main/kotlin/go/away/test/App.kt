package go.away.test

import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

@Serializable
data class Whatever(val x: Boolean)

fun main() {
    val input = "{\"x\": true}"
    val output = Json.encodeToString(Json.decodeFromString<Whatever>(input))
    println(output)
}
