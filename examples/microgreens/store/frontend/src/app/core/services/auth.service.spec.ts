import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { provideHttpClient } from '@angular/common/http';
import { TestBed } from '@angular/core/testing';
import { environment } from '../../../environments/environment';
import { AuthService } from './auth.service';

describe('AuthService', () => {
  let service: AuthService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    localStorage.clear();
    TestBed.configureTestingModule({
      providers: [AuthService, provideHttpClient(), provideHttpClientTesting()],
    });
    service = TestBed.inject(AuthService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
    localStorage.clear();
  });

  it('starts logged out with no stored session', () => {
    expect(service.isLoggedIn()).toBeFalse();
    expect(service.isAdmin()).toBeFalse();
    expect(service.token).toBeNull();
  });

  it('stores the session and flips isLoggedIn on successful login', () => {
    service.login('shopper@store.test', 'password123').subscribe();

    const req = httpMock.expectOne(`${environment.apiUrl}/auth/login`);
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ email: 'shopper@store.test', password: 'password123' });
    req.flush({ token: 'abc.def.ghi', email: 'shopper@store.test', role: 'customer' });

    expect(service.isLoggedIn()).toBeTrue();
    expect(service.isAdmin()).toBeFalse();
    expect(service.email()).toBe('shopper@store.test');
    expect(service.token).toBe('abc.def.ghi');
  });

  it('recognizes an admin role from a register response', () => {
    service.register('admin@store.test', 'password123').subscribe();

    const req = httpMock.expectOne(`${environment.apiUrl}/auth/register`);
    req.flush({ token: 'token', email: 'admin@store.test', role: 'admin' });

    expect(service.isAdmin()).toBeTrue();
  });

  it('persists the session across a fresh service instance (page reload)', () => {
    service.login('persist@store.test', 'password123').subscribe();
    httpMock
      .expectOne(`${environment.apiUrl}/auth/login`)
      .flush({ token: 'persisted-token', email: 'persist@store.test', role: 'customer' });

    TestBed.resetTestingModule();
    TestBed.configureTestingModule({
      providers: [AuthService, provideHttpClient(), provideHttpClientTesting()],
    });
    const reloaded = TestBed.inject(AuthService);
    expect(reloaded.token).toBe('persisted-token');
    expect(reloaded.isLoggedIn()).toBeTrue();

    // Re-establish the module the outer afterEach expects to verify against.
    httpMock = TestBed.inject(HttpTestingController);
  });

  it('clears the session on logout', () => {
    service.login('gone@store.test', 'password123').subscribe();
    httpMock
      .expectOne(`${environment.apiUrl}/auth/login`)
      .flush({ token: 'token', email: 'gone@store.test', role: 'customer' });

    service.logout();

    expect(service.isLoggedIn()).toBeFalse();
    expect(service.token).toBeNull();
    expect(localStorage.getItem('microgreens_auth')).toBeNull();
  });
});
