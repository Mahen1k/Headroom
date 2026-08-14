import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';

export interface AddBatchRequest {
  variety: string;
  sow_date: string;
  quantity_trays: number;
  price_per_tray: number;
}

export interface InspectRequest {
  height_cm: number;
  mold: boolean;
  color_uniform: boolean;
  stem_ok: boolean;
  root_mat_ok: boolean;
}

@Injectable({ providedIn: 'root' })
export class AdminService {
  constructor(private http: HttpClient) {}

  addBatch(req: AddBatchRequest): Observable<{ id: number }> {
    return this.http.post<{ id: number }>(`${environment.apiUrl}/admin/batches`, req);
  }

  inspectBatch(batchId: number, req: InspectRequest): Observable<{ updated: boolean }> {
    return this.http.post<{ updated: boolean }>(
      `${environment.apiUrl}/admin/batches/${batchId}/inspect`,
      req,
    );
  }
}
