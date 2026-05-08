"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.isBehaviorAllowed = exports.loadPolicy = void 0;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const semver_1 = __importDefault(require("semver"));
function loadPolicy(packageName) {
    const policiesDir = path.join(__dirname, '../../policies');
    const policyPath = path.join(policiesDir, `${packageName}.json`);
    if (!fs.existsSync(policyPath)) {
        return null;
    }
    try {
        const content = JSON.parse(fs.readFileSync(policyPath, 'utf-8'));
        return content;
    }
    catch {
        return null;
    }
}
exports.loadPolicy = loadPolicy;
function isBehaviorAllowed(profile, version, event) {
    if (!semver_1.default.satisfies(version, profile.versions)) {
        return false;
    }
    if (!profile.maintainers_verified)
        return false;
    // Check network events against allowed hosts
    if (event.event === 'connect' && profile.expected_behavior.network) {
        const host = extractHost(event.args);
        return profile.expected_behavior.network.allowed_hosts.some(h => host?.includes(h));
    }
    // Check filesystem writes against allowed paths
    if (event.event === 'fs_write_attempt' && event.path && profile.expected_behavior.filesystem) {
        return profile.expected_behavior.filesystem.writes.some(w => event.path?.includes(w));
    }
    return false;
}
exports.isBehaviorAllowed = isBehaviorAllowed;
function extractHost(args) {
    if (typeof args === 'string')
        return args;
    if (Array.isArray(args)) {
        const found = args.find(arg => typeof arg === 'string' && /https?:\/\//.test(arg));
        if (found && typeof found === 'string') {
            try {
                return new URL(found).hostname;
            }
            catch {
                return found;
            }
        }
    }
    return null;
}
