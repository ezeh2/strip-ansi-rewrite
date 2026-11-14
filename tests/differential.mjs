import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import stripAnsi from 'strip-ansi';

const binary = process.platform === 'win32'

  ? 'target/debug/strip-ansi.exe'
  : 'target/debug/strip-ansi';

const cases = [
  '',
  'plain text',
  'hello\nworld',
  '世界 🌍',
  '\u001B[31mred\u001B[0m',
  '\u001B[1;38;2;10;20;30mstyled\u001B[22;39m',
  '\u009B31mred\u009B0m',
  '\u001B[0m\u009D0;title\u0007visible',
  '\u001B[0m\u0090opaque\u009Cvisible',
  '\u001B]8;;https://example.com\u0007link\u001B]8;;\u0007',
  '\u001B]0;window title\u0007visible',
  '\u001BPopaque payload\u001B\\visible',
  '\u001B]P1234567visible',
  '\u001B]Rvisible',
  '\u001B7saved\u001B8restored',
  '\u001B(0line drawing\u001B#8alignment',
  '\u001B=application\u001B>normal',
  '\u001B[>0;276;0cresponse',
  '\u001B^private payload\u001B\\visible',
  '\u001B_kitty payload\u001B\\visible',
  '\u001B]8;;https://example.com\u001B\\link',
  'prefix\u001B[31mred\u001B[0msuffix',
  'literal [31m text',
  '\u001B[?25lhidden cursor\u001B[?25h',
  'before\u001B[31',
  'before\u001B]unterminated after',
  '\u001B[31mred\u001B[0m and \u001B[32mgreen\u001B[0m',
];

for (const parameter of ['', '0', '1', '1;31', '?25', '>0;276;0']) {
  for (const final of ['A', 'm', 'l', '~']) {
    cases.push(`before\u001B[${parameter}${final}after`);
  }
}

for (const payload of ['', 'title', '0;https://example.com', 'payload with spaces']) {
  cases.push(`before\u001B]${payload}\u0007after`);
  cases.push(`before\u001B]${payload}\u001B\\after`);
}

for (const prefix of ['P', 'X', '^', '_', 'k']) {
  cases.push(`before\u001B${prefix}payload\u001B\\after`);
  cases.push(`before\u001B${prefix}payload\u009Cafter`);
}

for (const input of cases) {
  const expected = stripAnsi(input);
  const actual = execFileSync(binary, [], {input, encoding: 'utf8'});
  assert.equal(actual, expected, `Mismatch for ${JSON.stringify(input)}`);
}

console.log(`Compared ${cases.length} cases with strip-ansi@7.2.0: all matched`);
