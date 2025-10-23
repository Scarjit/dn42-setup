import {
  InitRequestSchema,
  InitResponseSchema,
  VerifyRequestSchema,
  VerifyResponseSchema,
  DeployRequestSchema,
  DeployResponseSchema,
  ConfigResponseSchema,
  UpdateRequestSchema,
  UpdateResponseSchema,
  type InitRequest,
  type InitResponse,
  type VerifyRequest,
  type VerifyResponse,
  type DeployRequest,
  type DeployResponse,
  type ConfigResponse,
  type UpdateRequest,
  type UpdateResponse,
  type ApiError,
} from './types';

const API_BASE = '/peering';

class AutoPeerClient {
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error: ApiError = {
        message: await response.text(),
        status: response.status,
      };
      throw error;
    }

    return response.json();
  }

  async initPeering(data: InitRequest): Promise<InitResponse> {
    const validated = InitRequestSchema.parse(data);
    const response = await this.request<unknown>('/init', {
      method: 'POST',
      body: JSON.stringify(validated),
    });
    return InitResponseSchema.parse(response);
  }

  async verifyPeering(data: VerifyRequest): Promise<VerifyResponse> {
    const validated = VerifyRequestSchema.parse(data);
    const response = await this.request<unknown>('/verify', {
      method: 'POST',
      body: JSON.stringify(validated),
    });
    return VerifyResponseSchema.parse(response);
  }

  async deployPeering(data: DeployRequest, token: string): Promise<DeployResponse> {
    const validated = DeployRequestSchema.parse(data);
    const response = await this.request<unknown>('/deploy', {
      method: 'POST',
      body: JSON.stringify(validated),
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    return DeployResponseSchema.parse(response);
  }

  async getConfig(token: string): Promise<ConfigResponse> {
    const response = await this.request<unknown>('/config', {
      method: 'GET',
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    return ConfigResponseSchema.parse(response);
  }

  async updatePeering(data: UpdateRequest, token: string): Promise<UpdateResponse> {
    const validated = UpdateRequestSchema.parse(data);
    const response = await this.request<unknown>('/update', {
      method: 'PATCH',
      body: JSON.stringify(validated),
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    return UpdateResponseSchema.parse(response);
  }

  async deletePeering(token: string): Promise<UpdateResponse> {
    const response = await this.request<unknown>('', {
      method: 'DELETE',
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    return UpdateResponseSchema.parse(response);
  }
}

export const apiClient = new AutoPeerClient();
