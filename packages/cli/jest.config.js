module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/src'],
  testMatch: ['**/__tests__/**/*.test.ts', '**/?(*.)test.ts'],
  moduleNameMapper: {
    '^@guardinstall/policy-engine$': '<rootDir>/../../policy-engine/dist'
  }
}
