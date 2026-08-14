import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { HttpClient, provideHttpClient, withInterceptors } from '@angular/common/http';
import { TestBed } from '@angular/core/testing';
import { environment } from '../../../environments/environment';
import { AuthService } from '../services/auth.service';
import { authInterceptor } from './auth.interceptor';

describe('authInterceptor', () => {
  let http: HttpClient;
  let httpMock: HttpTestingController;
  let auth: AuthService;

  beforeEach(() => {
    localStorage.clear();
    TestBed.configureTestingModule({
      providers: [
        provideHttpClient(withInterceptors([authInterceptor])),
        provideHttpClientTesting(),
      ],
    });
    http = TestBed.inject(HttpClient);
    httpMock = TestBed.inject(HttpTestingController);
    auth = TestBed.inject(AuthService);
  });

  afterEach(() => {
    httpMock.verify();
    localStorage.clear();
  });

  it('attaches a bearer token when logged in', () => {
    auth.login('user@store.test', 'password123').subscribe();
    httpMock
      .expectOne(`${environment.apiUrl}/auth/login`)
      .flush({ token: 'my-token', email: 'user@store.test', role: 'customer' });

    http.get(`${environment.apiUrl}/products`).subscribe();
    const req = httpMock.expectOne(`${environment.apiUrl}/products`);
    expect(req.request.headers.get('Authorization')).toBe('Bearer my-token');
  });

  it('does not attach a header when logged out', () => {
    http.get(`${environment.apiUrl}/products`).subscribe();
    const req = httpMock.expectOne(`${environment.apiUrl}/products`);
    expect(req.request.headers.has('Authorization')).toBeFalse();
  });
});
