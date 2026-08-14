import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { adminGuard, authGuard } from './auth.guard';

describe('auth guards', () => {
  beforeEach(() => {
    localStorage.clear();
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting(), provideRouter([])],
    });
  });

  afterEach(() => localStorage.clear());

  it('authGuard blocks anonymous users and redirects to /login', () => {
    const result = TestBed.runInInjectionContext(() => authGuard({} as any, {} as any));
    expect(result).not.toBe(true);
    expect((result as any).toString()).toContain('/login');
  });

  it('authGuard allows logged-in users through', () => {
    localStorage.setItem(
      'microgreens_auth',
      JSON.stringify({ token: 't', email: 'u@store.test', role: 'customer' }),
    );
    TestBed.resetTestingModule();
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting(), provideRouter([])],
    });

    const result = TestBed.runInInjectionContext(() => authGuard({} as any, {} as any));
    expect(result).toBeTrue();
  });

  it('adminGuard blocks non-admins and redirects home', () => {
    localStorage.setItem(
      'microgreens_auth',
      JSON.stringify({ token: 't', email: 'u@store.test', role: 'customer' }),
    );
    TestBed.resetTestingModule();
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting(), provideRouter([])],
    });

    const result = TestBed.runInInjectionContext(() => adminGuard({} as any, {} as any));
    expect(result).not.toBe(true);
  });

  it('adminGuard allows admins through', () => {
    localStorage.setItem(
      'microgreens_auth',
      JSON.stringify({ token: 't', email: 'admin@store.test', role: 'admin' }),
    );
    TestBed.resetTestingModule();
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting(), provideRouter([])],
    });

    const result = TestBed.runInInjectionContext(() => adminGuard({} as any, {} as any));
    expect(result).toBeTrue();
  });
});
