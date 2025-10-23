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
});

export const VerifyResponseSchema = z.object({
  token: z.string(),
});

export const DeploymentInfoSchema = z.object({
  interface_address: z.string(),
  listen_port: z.number(),
  our_public_key: z.string(),
  our_endpoint: z.string(),
  bgp_neighbor: z.string(),
  bgp_local_as: z.number(),
  bgp_remote_as: z.number(),
  is_active: z.boolean(),
});

export type VerifyRequest = z.infer<typeof VerifyRequestSchema>;
export type VerifyResponse = z.infer<typeof VerifyResponseSchema>;
export type DeploymentInfo = z.infer<typeof DeploymentInfoSchema>;

// Deploy Peering
export const DeployRequestSchema = z.object({
  wg_public_key: z.string(),
  endpoint: z.string(),
});

export const DeployResponseSchema = z.object({
  deployment: DeploymentInfoSchema,
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
