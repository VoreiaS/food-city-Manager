import { apiClient } from "./client";

export interface Review {
  id: string;
  order_id: string;
  customer_id: string;
  restaurant_id: string;
  rating_food: number;
  rating_delivery: number;
  rating_packaging: number;
  rating_overall: number;
  body: string | null;
  photo_urls: string[];
  reply_body: string | null;
  reply_at: string | null;
  is_hidden: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateReviewInput {
  order_id: string;
  rating_food: number;
  rating_delivery: number;
  rating_packaging: number;
  rating_overall: number;
  body?: string;
  photo_urls?: string[];
}

export const reviewsApi = {
  create: async (input: CreateReviewInput) => {
    const { data } = await apiClient.post<Review>("/reviews", input);
    return data;
  },
  forRestaurant: async (restaurantId: string, page = 1, pageSize = 20) => {
    const { data } = await apiClient.get<Review[]>(`/restaurants/${restaurantId}/reviews`, {
      params: { page, page_size: pageSize },
    });
    return data;
  },
  reply: async (reviewId: string, reply: string) => {
    const { data } = await apiClient.post<Review>(`/reviews/${reviewId}/reply`, { reply });
    return data;
  },
};
