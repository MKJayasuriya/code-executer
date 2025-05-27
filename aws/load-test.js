import http from "k6/http";
import { check, sleep } from "k6";

export let options = {
    vus: 3, // 30 virtual users
    duration: "1m",
};

const BASE_URL = __ENV.BASE_URL || 'http://0.0.0.0:3000';

const tests = [
    {
        language: "python",
        code: `
def fib(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a

print(fib(20))  # Should print 6765
    `.trim(),
        expected: "6765"
    },
    {
        language: "javascript",
        code: `
function fib(n){ let a=0,b=1; for(let i=0;i<n;i++){ [a,b] = [b,a+b]; } return a; }
console.log(fib(20)); // Should print 6765
    `.trim(),
        expected: "6765"
    },
    {
        language: "java",
        code: `
public class Main {
    public static void main(String[] args) {
        System.out.println(fib(20));
    }
    static int fib(int n) {
        int a = 0, b = 1;
        for(int i = 0; i < n; i++) {
            int tmp = a;
            a = b;
            b = tmp + b;
        }
        return a;
    }
}
    `.trim(),
        expected: "6765"
    },
    {
        language: "cpp",
        code: `
#include <iostream>
using namespace std;

int fib(int n) {
    int a = 0, b = 1;
    for(int i = 0; i < n; i++) {
        int tmp = a;
        a = b;
        b = tmp + b;
    }
    return a;
}

int main() {
    cout << fib(20) << endl;
    return 0;
}
    `.trim(),
        expected: "6765"
    }
];

export default function () {
    for (const test of tests) {
        const payload = JSON.stringify({
            language: test.language,
            code: test.code,
        });

        const headers = {
            "Content-Type": "application/json",
        };

        const res = http.post(`${BASE_URL}/execute`, payload, { headers });

        const passed = check(res, {
            [`${test.language}: status is 200`]: (r) => r.status === 200,
            [`${test.language}: output is correct`]: (r) => {
                try {
                    const parsed = JSON.parse(r.body);
                    // Check both stdout and stderr for the expected output
                    return (
                        (parsed.stdout && parsed.stdout.toString().includes(test.expected)) ||
                        (parsed.stderr && parsed.stderr.toString().includes(test.expected))
                    );
                } catch (e) {
                    return false;
                }
            },
        });

        if (!passed) {
            console.log(`❌ Failed test for language: ${test.language}`);
            console.log(`Code:\n${test.code}`);
            console.log(`Response status: ${res.status}`);
            console.log(`Expected output: ${test.expected}`);
            console.log(`Actual response:\n${res.body}`);
        }

        sleep(1);
    }
}
