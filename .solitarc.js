const path = require('path');

module.exports = {
  idlGenerator: 'anchor',
  programName: 'meteora_fee_router',
  programId: 'HNgumZPoZAt5JmuqWCe2WRTPfP6MZcZgFTpYLUVkusWu',
  idlDir: path.join(__dirname, 'target', 'idl'),
  sdkDir: path.join(__dirname, 'src', 'generated'),
  binaryInstallDir: path.join(__dirname, '.crates'),
  programDir: path.join(__dirname, 'programs', 'meteora-fee-router'),
  anchorRemainingAccounts: false,
  formatCode: true,
  removeExistingIdl: false,
};
