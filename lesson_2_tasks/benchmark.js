/// <reference lib="es2020" />
// takes in two bigints 
function main() {
    console.log("1000000000 * 123456789101112");
    console.log(karatsuba(BigInt(10000000000), BigInt(123456789101112)));
    console.log("1234567891011121314151617 * 314159265358979");
    console.log(karatsuba(BigInt(1234567891011121314151617), BigInt(314159265358979)));
}
function karatsuba(a, b) {
    if (a < 10 || b < 10) {
        return a * b;
    }
    var a_size = a.toString().length;
    var b_size = b.toString().length;
    var splitidx = Math.floor(Math.max(a_size, b_size) / 2);
    var a_high = BigInt(a.toString().substring(0, a_size - splitidx));
    var a_low = BigInt(a.toString().substring(a_size - splitidx));
    var b_high = BigInt(b.toString().substring(0, b_size - splitidx));
    var b_low = BigInt(b.toString().substring(b_size - splitidx));
    var z0 = karatsuba(a_low, b_low);
    var z1 = karatsuba(a_low + a_high, b_low + b_high);
    var z2 = karatsuba(a_high, b_high);
    return (times_ten(z2.toString(), 2 * splitidx)) + times_ten((z1 - z2 - z0).toString(), splitidx) + z0;
}
function times_ten(a, pow) {
    var zeros = "";
    while (pow > 0) {
        pow--;
        zeros += "0";
    }
    return BigInt(a + zeros);
}
main();
