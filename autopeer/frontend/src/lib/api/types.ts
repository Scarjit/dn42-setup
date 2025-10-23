import { z } from 'zod';

// Init Peering
export const InitRequestSchema = z.object({
  asn: z.number().int().min(4242420000).max(4242423999),
});

export const InitResponseSchema = z.object({
  challenge: z.string(),
  pgp_fingerprint: z.string(),
});

export type InitRequest = z.infer<typeof InitRequestSchema>;
export type InitResponse = z.infer<typeof InitResponseSchema>;

// Verify Peering
export const VerifyRequestSchema = z.object({
  asn: z.number().int(),
  signed_challenge: z.string(),
  public_key: z.string(),
  wg_public_key: z.string(),
  endpoint: z.string(),
});

export const VerifyResponseSchema = z.object({
  token: z.string(),
  peer_public_key: z.string(),
  wireguard_config: z.string(),
});

export type VerifyRequest = z.infer<typeof VerifyRequestSchema>;
export type VerifyResponse = z.infer<typeof VerifyResponseSchema>;

// Deploy Peering
export const DeployRequestSchema = z.object({
  asn: z.number().int(),
});

export const DeployResponseSchema = z.object({
  status: z.string(),
  interface: z.string(),
});

export type DeployRequest = z.infer<typeof DeployRequestSchema>;
export type DeployResponse = z.infer<typeof DeployResponseSchema>;

// Config
export const ConfigResponseSchema = z.object({
  wireguard_config: z.string(),
});

export type ConfigResponse = z.infer<typeof ConfigResponseSchema>;

// Update Peering
export const UpdateRequestSchema = z.object({
  endpoint: z.string().optional(),
});

export const UpdateResponseSchema = z.object({
  status: z.string(),
});

export type UpdateRequest = z.infer<typeof UpdateRequestSchema>;
export type UpdateResponse = z.infer<typeof UpdateResponseSchema>;

// Error response
export interface ApiError {
  message: string;
  status: number;
}
