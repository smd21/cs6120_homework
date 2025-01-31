/// <reference lib="es2020" />
// takes in two bigints 

function karat_main(a: bigint, b: bigint): void {
  console.log(karatsuba(a, b))

}

function karatsuba(a: bigint, b: bigint): bigint {
  if (a < 10 || b < 10) {
    return a * b
  }
  var a_size: number = a.toString().length
  var b_size: number = b.toString().length

  var max: number = a_size >= b_size ? a_size : b_size
  var splitidx: number = max % 2 == 0 ? max / 2 : (max + 1) / 2

  var a_high: bigint = BigInt(a.toString().substring(0, a_size - splitidx))
  var a_low: bigint = BigInt(a.toString().substring(a_size - splitidx))
  var b_high: bigint = BigInt(b.toString().substring(0, b_size - splitidx))
  var b_low: bigint = BigInt(b.toString().substring(b_size - splitidx))

  var z0: bigint = karatsuba(a_low, b_low)
  var z1: bigint = karatsuba(a_low + a_high, b_low + b_high)
  var z2: bigint = karatsuba(a_high, b_high)

  return (times_ten(z2.toString(), 2 * splitidx)) + times_ten((z1 - z2 - z0).toString(), splitidx) + z0

}

function times_ten(a: string, pow: number): bigint {
  var zeros: string = ""
  while (pow > 0) {
    pow--
    zeros += "0"
  }
  return BigInt(a + zeros)
}