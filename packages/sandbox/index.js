// Load native addon - napi-rs output
const { sandboxProcess, getPlatform } = require('./index.node');

// Export functions
module.exports = {
  sandboxProcess,
  getPlatform,
};
