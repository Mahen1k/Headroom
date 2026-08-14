import { HttpClient } from '@angular/common/http';
import { Injectable, computed, signal } from '@angular/core';
import { Observable, tap } from 'rxjs';
import { environment } from '../../../environments/environment';
import { AuthResponse } from '../models/models';

const STORAGE_KEY = 'microgreens_auth';

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly authState = signal<AuthResponse | null>(this.readStoredAuth());

  readonly isLoggedIn = computed(() => this.authState() !== null);
  readonly isAdmin = computed(() => this.authState()?.role === 'admin');
  readonly email = computed(() => this.authState()?.email ?? null);

  constructor(private http: HttpClient) {}

  register(email: string, password: string): Observable<AuthResponse> {
    return this.http
      .post<AuthResponse>(`${environment.apiUrl}/auth/register`, { email, password })
      .pipe(tap((res) => this.storeAuth(res)));
  }

  login(email: string, password: string): Observable<AuthResponse> {
    return this.http
      .post<AuthResponse>(`${environment.apiUrl}/auth/login`, { email, password })
      .pipe(tap((res) => this.storeAuth(res)));
  }

  logout(): void {
    localStorage.removeItem(STORAGE_KEY);
    this.authState.set(null);
  }

  get token(): string | null {
    return this.authState()?.token ?? null;
  }

  private storeAuth(res: AuthResponse): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(res));
    this.authState.set(res);
  }

  private readStoredAuth(): AuthResponse | null {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as AuthResponse;
    } catch {
      return null;
    }
  }
}
