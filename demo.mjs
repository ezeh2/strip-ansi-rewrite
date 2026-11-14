import stripAnsi from 'strip-ansi';

const input = process.argv.slice(2).join(' ') ||
  '\u001b[32m✔ success\u001b[0m  \u001b[33mwarning\u001b[0m  \u001b[31merror\u001b[0m';

console.log('With ANSI styling:');
console.log(input);

console.log('\nAfter strip-ansi:');
console.log(stripAnsi(input));
