# 🚀 Meteora Fee Router - Backend API Documentation

## 📋 Overview

Complete REST API specification for the Meteora Fee Router backend. This document outlines all endpoints required for full integration with the deployed smart contract.

**🔗 Program ID:** `Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE`
**🌐 Network:** Solana Devnet
**📦 Base URL:** `https://api.yourdomain.com/api/v1`

## 🏗️ API Structure

### Base URL
```
https://api.yourdomain.com/api/v1
```

### Authentication
All endpoints require wallet signature authentication or API key authorization.

### Response Format
```json
{
  "success": true,
  "data": {},
  "message": "Operation completed successfully",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Error Format
```json
{
  "success": false,
  "error": {
    "code": 6000,
    "message": "Base fees detected - only quote fees allowed"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

---

## 📊 Policy Management Endpoints

### 1. Initialize Policy
**POST** `/policy/initialize`

Creates the main policy configuration for fee distribution.

**Request Body:**
```json
{
  "investorFeeShareBps": 500,
  "dailyCapLamports": 1000000,
  "minPayoutLamports": 10000,
  "y0TotalAllocation": 100000000,
  "quoteMint": "So11111111111111111111111111111111111111112"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "policyState": "PolicyPDA_Address",
    "transactionSignature": "5xKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxK"
  }
}
```

### 2. Get Policy State
**GET** `/policy/state`

Retrieves current policy configuration and state.

**Query Parameters:**
- `quoteMint` (optional): Filter by specific quote mint

**Response:**
```json
{
  "success": true,
  "data": {
    "authority": "Authority_PublicKey",
    "quoteMint": "Quote_Mint_Address",
    "investorFeeShareBps": 500,
    "dailyCapLamports": 1000000,
    "minPayoutLamports": 10000,
    "y0TotalAllocation": 100000000,
    "bump": 255
  }
}
```

### 3. Update Policy
**PUT** `/policy/update`

Updates policy parameters (admin only).

**Request Body:**
```json
{
  "investorFeeShareBps": 750,
  "dailyCapLamports": 2000000,
  "minPayoutLamports": 15000
}
```

---

## 🏦 Position Management Endpoints

### 4. Create Position
**POST** `/positions/create`

Creates a new honorary LP position.

**Request Body:**
```json
{
  "poolAddress": "Meteora_DLMM_Pool_Address",
  "quoteMint": "Quote_Mint_Address",
  "ownerWallet": "Owner_Wallet_Address"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "positionId": "Position_PDA_Address",
    "positionOwnerPda": "Position_Owner_PDA_Address",
    "transactionSignature": "5xKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxK"
  }
}
```

### 5. Get Position Details
**GET** `/positions/{positionId}`

Retrieves detailed information about a specific position.

**Path Parameters:**
- `positionId`: Position PDA address

**Response:**
```json
{
  "success": true,
  "data": {
    "owner": "Owner_PublicKey",
    "pool": "Meteora_Pool_Address",
    "quoteMint": "Quote_Mint_Address",
    "positionOwnerPda": "Position_Owner_PDA_Address",
    "claimedFees": 1250000,
    "createdAt": "2024-01-01T00:00:00Z",
    "lastClaim": "2024-01-02T00:00:00Z"
  }
}
```

### 6. List Positions
**GET** `/positions`

Retrieves all positions with optional filtering.

**Query Parameters:**
- `owner` (optional): Filter by owner wallet
- `quoteMint` (optional): Filter by quote mint
- `limit` (optional): Results per page (default: 50)
- `offset` (optional): Pagination offset

**Response:**
```json
{
  "success": true,
  "data": {
    "positions": [
      {
        "positionId": "Position_1_Address",
        "owner": "Owner_1_Address",
        "pool": "Pool_Address",
        "claimedFees": 1250000,
        "createdAt": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "totalPages": 1
  }
}
```

### 7. Get Positions by Owner
**GET** `/positions/by-owner/{ownerAddress}`

Retrieves all positions owned by a specific wallet.

**Path Parameters:**
- `ownerAddress`: Owner wallet address

---

## 💰 Fee Claiming Endpoints

### 8. Claim Fees
**POST** `/fees/claim/{positionId}`

Claims accumulated fees for a specific position.

**Path Parameters:**
- `positionId`: Position PDA address

**Request Body:**
```json
{
  "treasuryAta": "Treasury_Token_Account_Address"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "amountClaimed": 250000,
    "transactionSignature": "5xKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxK",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### 9. Get Claiming Status
**GET** `/fees/status/{positionId}`

Retrieves the current claiming status for a position.

**Response:**
```json
{
  "success": true,
  "data": {
    "positionId": "Position_Address",
    "totalClaimed": 1250000,
    "lastClaimAmount": 250000,
    "lastClaimTimestamp": "2024-01-01T00:00:00Z",
    "availableForClaim": 50000
  }
}
```

### 10. Get Claiming History
**GET** `/fees/history/{positionId}`

Retrieves the complete claiming history for a position.

**Query Parameters:**
- `limit` (optional): Results per page
- `offset` (optional): Pagination offset

**Response:**
```json
{
  "success": true,
  "data": {
    "history": [
      {
        "amount": 250000,
        "timestamp": "2024-01-01T00:00:00Z",
        "transactionSignature": "5xKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxK"
      }
    ],
    "totalClaims": 1,
    "totalAmount": 250000
  }
}
```

---

## 📈 Distribution System Endpoints

### 11. Start Daily Distribution
**POST** `/distribution/start`

Initiates a new 24-hour distribution cycle.

**Request Body:**
```json
{
  "distributionDay": 1704067200
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "distributionState": "Distribution_State_PDA",
    "distributionDay": 1704067200,
    "transactionSignature": "5xKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxK"
  }
}
```

### 12. Process Investor Page
**POST** `/distribution/process`

Processes a page of investors for the current distribution.

**Response:**
```json
{
  "success": true,
  "data": {
    "investorsProcessed": 50,
    "amountDistributed": 1000000,
    "remainingInvestors": 150,
    "transactionSignature": "5xKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxK"
  }
}
```

### 13. Complete Daily Distribution
**POST** `/distribution/complete`

Finalizes the current daily distribution cycle.

**Request Body:**
```json
{
  "creatorAta": "Creator_Token_Account_Address"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "totalDistributed": 5000000,
    "creatorRemainder": 500000,
    "transactionSignature": "5xKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxK"
  }
}
```

### 14. Get Distribution Status
**GET** `/distribution/status`

Retrieves the current distribution status.

**Response:**
```json
{
  "success": true,
  "data": {
    "distributionDay": 1704067200,
    "investorsProcessed": 150,
    "totalDistributedAmount": 5000000,
    "remainingAmount": 1000000,
    "isComplete": false,
    "startedAt": "2024-01-01T00:00:00Z"
  }
}
```

### 15. Get Distribution History
**GET** `/distribution/history`

Retrieves historical distribution data.

**Query Parameters:**
- `limit` (optional): Results per page
- `offset` (optional): Pagination offset
- `startDate` (optional): Start date filter
- `endDate` (optional): End date filter

**Response:**
```json
{
  "success": true,
  "data": {
    "distributions": [
      {
        "distributionDay": 1704067200,
        "totalDistributed": 5000000,
        "investorsProcessed": 150,
        "completedAt": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 1,
    "summary": {
      "totalDistributedOverall": 5000000,
      "averageDailyDistribution": 5000000
    }
  }
}
```

---

## 👥 Investor Management Endpoints

### 16. Get Investor Details
**GET** `/investors/{investorAddress}`

Retrieves detailed information about a specific investor.

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "Investor_Wallet_Address",
    "lockedTokens": 1000000,
    "totalReceived": 25000,
    "lastDistribution": "2024-01-01T00:00:00Z",
    "streamflowData": {
      "streamAddress": "Streamflow_Stream_Address",
      "lockedAmount": 1000000,
      "unlockDate": "2024-06-01T00:00:00Z"
    }
  }
}
```

### 17. List Investors
**GET** `/investors/list`

Retrieves all investors with their locked token information.

**Query Parameters:**
- `limit` (optional): Results per page
- `offset` (optional): Pagination offset
- `minLockedAmount` (optional): Filter by minimum locked amount

**Response:**
```json
{
  "success": true,
  "data": {
    "investors": [
      {
        "address": "Investor_1_Address",
        "lockedTokens": 1000000,
        "totalReceived": 25000,
        "percentage": 10.5
      }
    ],
    "total": 1,
    "summary": {
      "totalLockedTokens": 1000000,
      "averageLockAmount": 1000000
    }
  }
}
```

### 18. Get Distribution for Day
**GET** `/investors/distribution/{distributionDay}`

Retrieves distribution details for a specific day.

**Response:**
```json
{
  "success": true,
  "data": {
    "distributionDay": 1704067200,
    "totalDistributed": 5000000,
    "investors": [
      {
        "address": "Investor_Address",
        "amountReceived": 525000,
        "percentage": 10.5
      }
    ]
  }
}
```

### 19. Get Streamflow Data
**GET** `/investors/locked-tokens`

Retrieves all Streamflow locked token data for distribution calculations.

**Response:**
```json
{
  "success": true,
  "data": {
    "totalLockedTokens": 10000000,
    "investors": [
      {
        "address": "Investor_Address",
        "lockedAmount": 1000000,
        "streamAddress": "Stream_Address",
        "unlockDate": "2024-06-01T00:00:00Z"
      }
    ]
  }
}
```

---

## 📡 Monitoring & Events Endpoints

### 20. Get Events
**GET** `/events`

Retrieves recent system events.

**Query Parameters:**
- `limit` (optional): Number of events (default: 50)
- `offset` (optional): Pagination offset
- `eventType` (optional): Filter by event type
- `startDate` (optional): Start date filter

**Response:**
```json
{
  "success": true,
  "data": {
    "events": [
      {
        "type": "QuoteFeesClaimed",
        "data": {
          "amount": 250000,
          "timestamp": "2024-01-01T00:00:00Z"
        },
        "transactionSignature": "5xKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxKxK",
        "blockTime": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 1
  }
}
```

### 21. Subscribe to Events (WebSocket)
**GET** `/events/subscribe`

Establishes a WebSocket connection for real-time event streaming.

**Events Streamed:**
- `HonoraryPositionInitialized`
- `QuoteFeesClaimed`
- `InvestorsProcessed`
- `CreatorPayoutCompleted`

### 22. Get Metrics
**GET** `/metrics`

Retrieves system metrics and analytics.

**Response:**
```json
{
  "success": true,
  "data": {
    "totalPositions": 5,
    "totalClaimedFees": 1250000,
    "totalDistributions": 3,
    "totalDistributedAmount": 15000000,
    "activeDistributions": 1,
    "averageDailyDistribution": 5000000,
    "last24Hours": {
      "feesClaimed": 250000,
      "distributions": 1
    }
  }
}
```

### 23. Health Check
**GET** `/health`

Basic health check endpoint for monitoring.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-01T00:00:00Z",
    "version": "1.0.0",
    "network": "devnet",
    "programId": "Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE"
  }
}
```

---

## 🔧 Utility & Admin Endpoints

### 24. Get Program Info
**GET** `/program/info`

Retrieves general program information.

**Response:**
```json
{
  "success": true,
  "data": {
    "programId": "Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE",
    "network": "devnet",
    "binarySize": 459584,
    "lastDeployment": "2024-01-01T00:00:00Z",
    "authority": "Authority_PublicKey"
  }
}
```

### 25. Get Network Status
**GET** `/network/status`

Retrieves current network and configuration status.

**Response:**
```json
{
  "success": true,
  "data": {
    "network": "devnet",
    "rpcEndpoint": "https://api.devnet.solana.com",
    "currentSlot": 412134279,
    "blockTime": "2024-01-01T00:00:00Z",
    "programAccounts": 5
  }
}
```

### 26. Retry Transaction (Admin)
**POST** `/admin/retry-transaction`

Retries a failed transaction (admin only).

**Request Body:**
```json
{
  "transactionSignature": "Failed_Transaction_Signature",
  "reason": "Transaction failed due to network congestion"
}
```

---

## 📋 Summary

### Total Endpoints: 26

**Core Business Logic:** 15 endpoints
- Policy Management: 3
- Position Management: 4
- Fee Claiming: 3
- Distribution System: 5

**Supporting Features:** 11 endpoints
- Investor Management: 4
- Monitoring & Events: 4
- Utility & Admin: 3

### API Design Principles
- ✅ RESTful design patterns
- ✅ Consistent error handling
- ✅ Comprehensive response metadata
- ✅ Pagination support where needed
- ✅ Real-time event streaming
- ✅ Input validation and sanitization

This API provides complete backend functionality for the Meteora Fee Router, supporting all smart contract operations with proper error handling, monitoring, and scalability features.
